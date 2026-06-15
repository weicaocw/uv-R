//! 版本号：解析、比较、约束。
//!
//! 这是从 Step 01–07 的 `main.rs` 移过来的版本模型，现在作为库模块对外提供。

use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct Version {
    parts: Vec<u64>,
}

impl Version {
    /// 把像 "1.2.3"、"3.4-1" 的字符串解析成 Version；有一段不是数字则返回 None。
    pub fn parse(s: &str) -> Option<Version> {
        let mut parts = Vec::new();
        for piece in s.split(['.', '-']) {
            match piece.parse::<u64>() {
                Ok(number) => parts.push(number),
                Err(_) => return None,
            }
        }
        Some(Version { parts })
    }
}

// 逐段比较；较短的一方用 0 补齐，于是 1.0 == 1.0.0。
impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let n = self.parts.len().max(other.parts.len());
        for i in 0..n {
            let a = self.parts.get(i).copied().unwrap_or(0);
            let b = other.parts.get(i).copied().unwrap_or(0);
            match a.cmp(&b) {
                Ordering::Equal => continue,
                non_eq => return non_eq,
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// 相等性与 cmp 一致：a == b 当且仅当 cmp 为 Equal。
impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Version {}

// Hash 必须与 Eq 一致：相等的值哈希也必须相等（否则放进 HashMap 会出错）。
// 我们的 Eq 视 1.0 == 1.0.0，故哈希前先去掉尾部的 0，让它们哈希到同一个值。
// 这与 Step 05 "Eq/Ord 必须一致"是同一类契约。
impl std::hash::Hash for Version {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut end = self.parts.len();
        while end > 0 && self.parts[end - 1] == 0 {
            end -= 1;
        }
        std::hash::Hash::hash(&self.parts[..end], state);
    }
}

/// 用 `.` 连接各段渲染版本号（如 [1,2,0] → "1.2.0"）。`Display` 必须手写
/// （编译器不会替你猜"对用户友好的样子"），有了它 `{}` 与 `.to_string()` 才能用。
impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dotted = self
            .parts
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(".");
        write!(f, "{dotted}")
    }
}

/// 比较运算符。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Lt,
    Le,
    Eq,
    Ge,
    Gt,
}

/// 版本约束，如 ">= 1.2.0"。
#[derive(Debug, Clone)]
pub struct Constraint {
    op: Op,
    version: Version,
}

impl Constraint {
    /// 解析 ">= 1.2.0" 这类约束。先认两字符运算符，避免把 ">=" 误当 ">"。
    pub fn parse(s: &str) -> Option<Constraint> {
        let s = s.trim();
        let (op, rest) = if let Some(r) = s.strip_prefix(">=") {
            (Op::Ge, r)
        } else if let Some(r) = s.strip_prefix("<=") {
            (Op::Le, r)
        } else if let Some(r) = s.strip_prefix("==") {
            (Op::Eq, r)
        } else if let Some(r) = s.strip_prefix('>') {
            (Op::Gt, r)
        } else if let Some(r) = s.strip_prefix('<') {
            (Op::Lt, r)
        } else if let Some(r) = s.strip_prefix('=') {
            (Op::Eq, r)
        } else {
            return None;
        };
        Some(Constraint {
            op,
            version: Version::parse(rest.trim())?,
        })
    }

    /// 本约束的比较运算符。
    pub fn op(&self) -> Op {
        self.op
    }

    /// 本约束里的版本。
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// 给定版本是否满足本约束。
    pub fn matches(&self, v: &Version) -> bool {
        let ord = v.cmp(&self.version);
        match self.op {
            Op::Lt => ord == Ordering::Less,
            Op::Le => ord != Ordering::Greater,
            Op::Eq => ord == Ordering::Equal,
            Op::Ge => ord != Ordering::Less,
            Op::Gt => ord == Ordering::Greater,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dotted_version() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.parts, vec![1, 2, 3]);
    }

    #[test]
    fn rejects_non_numeric() {
        assert!(Version::parse("1.2.x").is_none());
    }

    #[test]
    fn orders_numerically_not_lexically() {
        let a = Version::parse("1.2.3").unwrap();
        let b = Version::parse("1.10.0").unwrap();
        assert!(a < b, "1.2.3 应当小于 1.10.0");
    }

    #[test]
    fn trailing_zeros_are_equal() {
        assert_eq!(
            Version::parse("1.0").unwrap(),
            Version::parse("1.0.0").unwrap()
        );
    }

    #[test]
    fn constraint_ge_matches() {
        let c = Constraint::parse(">= 1.2.0").unwrap();
        assert!(c.matches(&Version::parse("1.10.0").unwrap()));
        assert!(!c.matches(&Version::parse("1.1.9").unwrap()));
    }

    #[test]
    fn constraint_lt_uses_zero_padding() {
        let c = Constraint::parse("< 2.0").unwrap();
        assert!(c.matches(&Version::parse("1.9.9").unwrap()));
        assert!(!c.matches(&Version::parse("2.0.0").unwrap()));
    }
}
