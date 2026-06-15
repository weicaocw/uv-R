//! lockfile：把求解结果写成确定性文本，并能读回（往返）。
//!
//! 这是为"可复现安装"准备的落点：锁定每个包的确切版本**与来源仓库**。格式刻意简单（纯 std）：
//! 一行一个 `name version [repo]`，按包名排序，开头一行注释。
//! **v2** 多记一个来源仓库，使 `sync` 能自包含（无需再传 `--repo`）；
//! 解析兼容 **v1**（只有 `name version` 两列，repo 视为空）。
//! 生产工具常用 TOML/JSON（需 serde 等 crate），这里先用最小可用的文本格式。

use crate::version::Version;
use std::collections::BTreeMap;

const HEADER: &str = "# uvr lockfile v2";

/// 一个被锁定的包：确切版本 + 来源仓库（v1 锁文件里 repo 为空）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locked {
    pub version: Version,
    pub repo: String,
}

/// 把"包名 → 锁定项"渲染成 lockfile 文本（确定性：按包名排序，由 BTreeMap 保证）。
///
/// 有来源仓库就写三列 `name version repo`；没有（如本地 PACKAGES）则退化为两列 `name version`。
pub fn render(resolution: &BTreeMap<String, Locked>) -> String {
    let mut out = String::from(HEADER);
    out.push('\n');
    for (name, lk) in resolution {
        if lk.repo.is_empty() {
            out.push_str(&format!("{name} {}\n", lk.version));
        } else {
            out.push_str(&format!("{name} {} {}\n", lk.version, lk.repo));
        }
    }
    out
}

/// 把 lockfile 文本解析回"包名 → 锁定项"。忽略空行与 `#` 注释；
/// 两列按 v1（repo 为空）解释、三列按 v2 解释；其它列数 / 版本非法则返回 None。
pub fn parse(text: &str) -> Option<BTreeMap<String, Locked>> {
    let mut map = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        let (name, ver, repo) = match parts.as_slice() {
            [n, v] => (*n, *v, ""),    // v1：两列
            [n, v, r] => (*n, *v, *r), // v2：三列
            _ => return None,
        };
        map.insert(
            name.to_string(),
            Locked {
                version: Version::parse(ver)?,
                repo: repo.to_string(),
            },
        );
    }
    Some(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locked(ver: &str, repo: &str) -> Locked {
        Locked {
            version: Version::parse(ver).unwrap(),
            repo: repo.to_string(),
        }
    }

    fn resolution() -> BTreeMap<String, Locked> {
        let mut res = BTreeMap::new();
        res.insert("pkgB".to_string(), locked("2.0.0", "https://r2"));
        res.insert("pkgA".to_string(), locked("1.2.0", "https://r1"));
        res
    }

    #[test]
    fn round_trips_v2_with_repos() {
        let res = resolution();
        let text = render(&res);
        assert!(text.starts_with("# uvr lockfile v2"));
        assert!(text.contains("pkgA 1.2.0 https://r1")); // 三列、按名排序
        let back = parse(&text).unwrap();
        assert_eq!(back, res); // 往返后内容一致（含仓库）
    }

    #[test]
    fn renders_two_columns_when_repo_empty() {
        let mut res = BTreeMap::new();
        res.insert("pkgA".to_string(), locked("1.0.0", ""));
        let text = render(&res);
        assert!(text.contains("pkgA 1.0.0\n"));
        assert!(!text.contains("pkgA 1.0.0 ")); // 没有多余的尾随空格 / 第三列
    }

    #[test]
    fn parses_v1_two_column_as_empty_repo() {
        // 兼容旧 v1 锁文件：两列 → repo 为空。
        let back = parse("# uvr lockfile v1\npkgA 1.2.0\n").unwrap();
        assert_eq!(back["pkgA"], locked("1.2.0", ""));
    }

    #[test]
    fn rejects_malformed_line() {
        assert!(parse("# uvr lockfile v2\nbadlineWithoutVersion\n").is_none());
        assert!(parse("a 1.0 repo extra\n").is_none()); // 四列也非法
    }
}
