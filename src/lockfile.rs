//! lockfile：把求解结果写成确定性文本，并能读回（往返）。
//!
//! 这是为"可复现安装"准备的落点：锁定每个包的确切版本、来源仓库**与校验和**。格式刻意简单（纯 std）：
//! 一行一个 `name version [repo [hash]]`，按包名排序，开头一行注释。
//! 版本演进：**v1** `name version` → **v2** 加来源仓库（`sync` 自包含）→ **v3** 再加校验和（完整性校验）。
//! 解析按**列数**兼容三代：2 列=v1、3 列=v2、4 列=v3。

use crate::version::Version;
use std::collections::BTreeMap;

const HEADER: &str = "# uvr lockfile v3";

/// 一个被锁定的包：确切版本 + 来源仓库 + 校验和（带算法前缀，如 `sha256:…`；缺则为空）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locked {
    pub version: Version,
    pub repo: String,
    pub hash: String,
}

/// 把"包名 → 锁定项"渲染成 lockfile 文本（确定性：按包名排序，由 BTreeMap 保证）。
///
/// 列数随可用信息伸缩：有校验和写四列、有仓库写三列、否则两列（校验和依附于仓库，不单独出现）。
pub fn render(resolution: &BTreeMap<String, Locked>) -> String {
    let mut out = String::from(HEADER);
    out.push('\n');
    for (name, lk) in resolution {
        if !lk.repo.is_empty() && !lk.hash.is_empty() {
            out.push_str(&format!("{name} {} {} {}\n", lk.version, lk.repo, lk.hash));
        } else if !lk.repo.is_empty() {
            out.push_str(&format!("{name} {} {}\n", lk.version, lk.repo));
        } else {
            out.push_str(&format!("{name} {}\n", lk.version));
        }
    }
    out
}

/// 把 lockfile 文本解析回"包名 → 锁定项"。忽略空行与 `#` 注释；
/// 按列数兼容 v1（2 列）/ v2（3 列）/ v3（4 列）；其它列数 / 版本非法则返回 None。
pub fn parse(text: &str) -> Option<BTreeMap<String, Locked>> {
    let mut map = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        let (name, ver, repo, hash) = match parts.as_slice() {
            [n, v] => (*n, *v, "", ""),       // v1
            [n, v, r] => (*n, *v, *r, ""),    // v2
            [n, v, r, h] => (*n, *v, *r, *h), // v3
            _ => return None,
        };
        map.insert(
            name.to_string(),
            Locked {
                version: Version::parse(ver)?,
                repo: repo.to_string(),
                hash: hash.to_string(),
            },
        );
    }
    Some(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locked(ver: &str, repo: &str, hash: &str) -> Locked {
        Locked {
            version: Version::parse(ver).unwrap(),
            repo: repo.to_string(),
            hash: hash.to_string(),
        }
    }

    #[test]
    fn round_trips_v3_with_hash() {
        let mut res = BTreeMap::new();
        res.insert(
            "pkgB".to_string(),
            locked("2.0.0", "https://r2", "sha256:bb"),
        );
        res.insert(
            "pkgA".to_string(),
            locked("1.2.0", "https://r1", "sha256:aa"),
        );
        let text = render(&res);
        assert!(text.starts_with("# uvr lockfile v3"));
        assert!(text.contains("pkgA 1.2.0 https://r1 sha256:aa"));
        assert_eq!(parse(&text).unwrap(), res); // 往返一致（含校验和）
    }

    #[test]
    fn renders_three_columns_when_hash_empty() {
        let mut res = BTreeMap::new();
        res.insert("pkgA".to_string(), locked("1.0.0", "https://r1", ""));
        assert!(render(&res).contains("pkgA 1.0.0 https://r1\n"));
    }

    #[test]
    fn parses_v2_three_column_as_empty_hash() {
        let back = parse("# v2\npkgA 1.2.0 https://r1\n").unwrap();
        assert_eq!(back["pkgA"], locked("1.2.0", "https://r1", ""));
    }

    #[test]
    fn parses_v1_two_column_as_empty_repo_and_hash() {
        let back = parse("pkgA 1.2.0\n").unwrap();
        assert_eq!(back["pkgA"], locked("1.2.0", "", ""));
    }

    #[test]
    fn rejects_malformed_line() {
        assert!(parse("badlineWithoutVersion\n").is_none());
        assert!(parse("a 1.0 repo hash extra\n").is_none()); // 5 列非法
    }
}
