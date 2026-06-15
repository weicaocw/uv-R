// Step 04：让 Version 能比较大小（按数字逐段比，而非字符串）。

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    parts: Vec<u64>,
}

impl Version {
    // 把像 "1.2.3"、"3.4-1" 的字符串解析成 Version；
    // 只要有一段不是数字，就返回 None（表示"解析失败"）。
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

fn main() {
    println!("{:?}", Version::parse("1.2.3")); // Some(Version { parts: [1, 2, 3] })
    println!("{:?}", Version::parse("1.2.x")); // None
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
}
