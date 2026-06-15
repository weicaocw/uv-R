//! 解析 R 的 DCF 格式（`PACKAGES` / `DESCRIPTION`）、依赖字段，并建立包索引。
//!
//! 数据流：DCF 文本 → 记录（字段→值）→ 依赖项 → 包索引（包名 → 各版本的包）。

use crate::version::{Constraint, Version};
use std::collections::BTreeMap;

/// 一条 DCF 记录：字段名 → 值。用 BTreeMap 以便迭代顺序稳定（测试友好）。
pub type Record = BTreeMap<String, String>;

/// 把 DCF 文本解析成若干记录。
pub fn parse(input: &str) -> Vec<Record> {
    let mut records = Vec::new();
    let mut current = Record::new();
    let mut last_key: Option<String> = None;

    for line in input.lines() {
        if line.trim().is_empty() {
            if !current.is_empty() {
                records.push(std::mem::take(&mut current));
            }
            last_key = None;
            continue;
        }

        if line.starts_with(' ') || line.starts_with('\t') {
            // 折行续行：用空格拼到上一字段（let-chain，Rust 2024）
            if let Some(key) = &last_key
                && let Some(val) = current.get_mut(key)
            {
                val.push(' ');
                val.push_str(line.trim());
            }
            continue;
        }

        if let Some(idx) = line.find(':') {
            let key = line[..idx].trim().to_string();
            let value = line[idx + 1..].trim().to_string();
            current.insert(key.clone(), value);
            last_key = Some(key);
        }
    }
    if !current.is_empty() {
        records.push(current);
    }
    records
}

/// 一条依赖：包名 + 可选版本约束。
#[derive(Debug)]
pub struct Dependency {
    pub name: String,
    pub constraint: Option<Constraint>,
}

/// 解析依赖字段，如 `"R (>= 2.15.0), xtable, pbapply (>= 1.3-2)"`。
pub fn parse_dependencies(field: &str) -> Vec<Dependency> {
    field
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(parse_one_dependency)
        .collect()
}

fn parse_one_dependency(entry: &str) -> Dependency {
    match (entry.find('('), entry.rfind(')')) {
        (Some(open), Some(close)) if close > open => Dependency {
            name: entry[..open].trim().to_string(),
            constraint: Constraint::parse(entry[open + 1..close].trim()),
        },
        _ => Dependency {
            name: entry.trim().to_string(),
            constraint: None,
        },
    }
}

/// 一个包（某个具体版本）：名字 + 版本 + 它依赖的其它包。
#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub depends: Vec<Dependency>,
}

impl Package {
    /// 从一条 DCF 记录构造一个包；缺少 `Package` / `Version` 则返回 None。
    /// 依赖来自 `Depends` / `Imports` / `LinkingTo` 三个字段的合并。
    fn from_record(rec: &Record) -> Option<Package> {
        let name = rec.get("Package")?.clone();
        let version = Version::parse(rec.get("Version")?)?;
        let mut depends = Vec::new();
        for field in ["Depends", "Imports", "LinkingTo"] {
            if let Some(value) = rec.get(field) {
                depends.extend(parse_dependencies(value));
            }
        }
        Some(Package {
            name,
            version,
            depends,
        })
    }
}

/// 包索引：按包名查它的（可能多个）版本。这是依赖求解的输入。
#[derive(Debug, Default)]
pub struct PackageIndex {
    by_name: BTreeMap<String, Vec<Package>>,
}

impl PackageIndex {
    /// 解析整份 `PACKAGES` 文本，建立索引。无法构造成包的记录被跳过。
    pub fn from_packages_file(input: &str) -> PackageIndex {
        let mut by_name: BTreeMap<String, Vec<Package>> = BTreeMap::new();
        for rec in parse(input) {
            if let Some(pkg) = Package::from_record(&rec) {
                by_name.entry(pkg.name.clone()).or_default().push(pkg);
            }
        }
        PackageIndex { by_name }
    }

    /// 某包名下的所有版本（没有则返回空切片）。
    pub fn versions_of(&self, name: &str) -> &[Package] {
        self.by_name.get(name).map(Vec::as_slice).unwrap_or(&[])
    }

    /// 遍历索引里的所有包（各包名下的各版本展平）。
    pub fn packages(&self) -> impl Iterator<Item = &Package> + '_ {
        self.by_name.values().flatten()
    }

    /// 索引里有多少个不同的包名。
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    /// 索引是否为空。
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
Package: A3
Version: 1.0.0
Depends: R (>= 2.15.0), xtable, pbapply
License: GPL (>= 2)

Package: aaSEA
Version: 1.1.0
Imports: DT (>= 0.4), networkD3 (>= 0.4),
        shiny (>= 1.0.5)
";

    #[test]
    fn parses_each_record() {
        let recs = parse(SAMPLE);
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0]["Package"], "A3");
        assert_eq!(recs[0]["Version"], "1.0.0");
        assert_eq!(recs[1]["Package"], "aaSEA");
    }

    #[test]
    fn folds_continuation_lines() {
        let recs = parse(SAMPLE);
        let imports = &recs[1]["Imports"];
        assert!(imports.contains("shiny (>= 1.0.5)"));
        assert!(!imports.contains('\n'));
    }

    #[test]
    fn ignores_trailing_blank_lines() {
        let recs = parse("Package: x\nVersion: 1.0\n\n\n");
        assert_eq!(recs.len(), 1);
    }

    #[test]
    fn parses_name_and_constraint() {
        let deps = parse_dependencies("R (>= 2.15.0), xtable, pbapply (>= 1.3-2)");
        assert_eq!(deps.len(), 3);
        assert_eq!(deps[0].name, "R");
        assert!(deps[0].constraint.is_some());
        assert_eq!(deps[1].name, "xtable");
        assert!(deps[1].constraint.is_none());
        assert_eq!(deps[2].name, "pbapply");
    }

    #[test]
    fn parsed_constraint_actually_matches() {
        let deps = parse_dependencies("R (>= 3.4.0)");
        let c = deps[0].constraint.as_ref().unwrap();
        assert!(c.matches(&Version::parse("4.5.2").unwrap()));
        assert!(!c.matches(&Version::parse("3.0.0").unwrap()));
    }

    #[test]
    fn builds_index_with_deps() {
        let idx = PackageIndex::from_packages_file(SAMPLE);
        assert_eq!(idx.len(), 2);

        let a3 = &idx.versions_of("A3")[0];
        assert_eq!(a3.version, Version::parse("1.0.0").unwrap());
        assert_eq!(a3.depends.len(), 3); // R, xtable, pbapply

        // aaSEA 的依赖来自折行的 Imports：DT, networkD3, shiny
        let aasea = &idx.versions_of("aaSEA")[0];
        assert_eq!(aasea.depends.len(), 3);

        assert!(idx.versions_of("nonexistent").is_empty());
    }
}
