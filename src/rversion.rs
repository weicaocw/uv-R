//! R 版本管理：发现系统里的 R、项目级钉版本（`.R-version`）、按优先级选择该用哪个 R。
//!
//! 对标 uv 的 `uv python` 家族：uvr 不自己编译 R（就像 uv 不自己编译 Python），
//! 只负责**发现 / 选择 / 使用**已安装的 R；**获取**一个新 R 是系统级操作，交给 `rig` 或交还用户。

use crate::version::Version;

/// 从 `R --version` 的输出里解析出版本号。
///
/// 典型首行：`R version 4.5.2 (2025-10-31) -- "..."`。
/// 取 `version` 之后紧跟的那个词（`4.5.2`），而不是括号里的日期（`2025-10-31`）。
pub fn parse_r_version(output: &str) -> Option<Version> {
    let mut tokens = output.split_whitespace();
    while let Some(tok) = tokens.next() {
        if tok == "version" {
            let candidate = tokens.next()?.trim_end_matches([',', ')']);
            return Version::parse(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_first_line() {
        let line = r#"R version 4.5.2 (2025-10-31) -- "[Not] Part in a Rumble""#;
        assert_eq!(parse_r_version(line), Version::parse("4.5.2"));
    }

    #[test]
    fn takes_version_not_the_date() {
        // 括号里的日期含 '-'，若误取就会变成 2025.10.31——确保我们取的是 4.5.2。
        let v = parse_r_version("R version 4.5.2 (2025-10-31) -- x").unwrap();
        assert_eq!(v, Version::parse("4.5.2").unwrap());
        assert_ne!(v, Version::parse("2025.10.31").unwrap());
    }

    #[test]
    fn parses_bare_version() {
        assert_eq!(parse_r_version("R version 3.6.1"), Version::parse("3.6.1"));
    }

    #[test]
    fn none_when_no_version_word() {
        assert_eq!(parse_r_version("R Under development (unstable)"), None);
        assert_eq!(parse_r_version("totally unrelated text"), None);
    }
}
