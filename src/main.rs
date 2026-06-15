// Step 05：修正版本比较——按 R 语义"零填充逐段比较"，
// 并让 ==（PartialEq）与 cmp（Ord）保持一致（Eq/Ord 契约）。

use std::cmp::Ordering;

#[derive(Debug)]
struct Version {
    parts: Vec<u64>,
}

impl Version {
    fn parse(s: &str) -> Option<Version> {
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

// 相等性必须与 cmp 一致：a == b 当且仅当 cmp 为 Equal。
// 否则把 Version 放进有序集合或排序时会行为错乱。
impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Version {}

fn main() {
    println!("{:?}", Version::parse("1.2.3"));
    println!("{:?}", Version::parse("1.2.x"));
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
        // R 语义：1.0 与 1.0.0 应当相等
        assert_eq!(
            Version::parse("1.0").unwrap(),
            Version::parse("1.0.0").unwrap()
        );
    }
}
