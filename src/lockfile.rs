//! lockfile：把求解结果写成确定性文本，并能读回（往返）。
//!
//! 这是为"可复现安装"准备的落点：锁定每个包的确切版本。格式刻意简单（纯 std）：
//! 一行一个 `name version`，按包名排序，开头一行注释。生产工具常用 TOML/JSON
//! （需 serde 等 crate），这里先用最小可用的文本格式。

use crate::version::Version;
use std::collections::BTreeMap;

const HEADER: &str = "# uvr lockfile v1";

/// 把"包名 → 版本"渲染成 lockfile 文本（确定性：按包名排序，由 BTreeMap 保证）。
pub fn render(resolution: &BTreeMap<String, Version>) -> String {
    let mut out = String::from(HEADER);
    out.push('\n');
    for (name, version) in resolution {
        out.push_str(&format!("{name} {version}\n"));
    }
    out
}

/// 把 lockfile 文本解析回"包名 → 版本"。忽略空行与 `#` 注释；格式非法则返回 None。
pub fn parse(text: &str) -> Option<BTreeMap<String, Version>> {
    let mut map = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, ver) = line.split_once(' ')?;
        map.insert(name.to_string(), Version::parse(ver)?);
    }
    Some(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolution() -> BTreeMap<String, Version> {
        let mut res = BTreeMap::new();
        res.insert("pkgB".to_string(), Version::parse("2.0.0").unwrap());
        res.insert("pkgA".to_string(), Version::parse("1.2.0").unwrap());
        res
    }

    #[test]
    fn round_trips() {
        let res = resolution();
        let text = render(&res);
        assert!(text.starts_with("# uvr lockfile v1"));
        assert!(text.contains("pkgA 1.2.0")); // Display 渲染 + 按名排序
        let back = parse(&text).unwrap();
        assert_eq!(back, res); // 往返后内容一致
    }

    #[test]
    fn rejects_malformed_line() {
        assert!(parse("# uvr lockfile v1\nbadlineWithoutVersion\n").is_none());
    }
}
