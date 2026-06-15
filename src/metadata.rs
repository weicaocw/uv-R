//! 解析 R 的 DCF 格式（`PACKAGES` / `DESCRIPTION`）与其依赖字段。
//!
//! 一个 DCF 文档由若干"记录"组成，记录之间用空行分隔；每条记录是若干
//! `字段: 值` 行；以空白开头的行是上一字段的"折行续行"，要拼回去。

use crate::version::Constraint;
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
            // 空行 = 记录分隔
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
        // "name (constraint)"：括号里是版本约束
        (Some(open), Some(close)) if close > open => Dependency {
            name: entry[..open].trim().to_string(),
            constraint: Constraint::parse(entry[open + 1..close].trim()),
        },
        // 只有包名，无版本约束
        _ => Dependency {
            name: entry.trim().to_string(),
            constraint: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::Version;

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
}
