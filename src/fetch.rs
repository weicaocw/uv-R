//! 联网抓取 R 仓库的 `PACKAGES`（用 ureq 做 HTTP GET）。
//!
//! 把"构造 URL"（纯函数、可测）与"实际联网"（IO、不在 CI 跑）分开。

use std::fmt;

/// 抓取失败的原因。把底层错误包成一条消息，不让 `ureq` 的类型泄漏进我们的 API。
#[derive(Debug)]
pub struct FetchError(pub String);

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 由仓库基址构造 `PACKAGES` 的 URL：`<base>/src/contrib/PACKAGES`。
pub fn packages_url(repo_base: &str) -> String {
    format!("{}/src/contrib/PACKAGES", repo_base.trim_end_matches('/'))
}

/// HTTP GET 一个 URL，返回正文文本。
pub fn get_text(url: &str) -> Result<String, FetchError> {
    ureq::get(url)
        .call()
        .map_err(|e| FetchError(e.to_string()))?
        .into_string()
        .map_err(|e| FetchError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_packages_url() {
        assert_eq!(
            packages_url("https://jeroen.r-universe.dev"),
            "https://jeroen.r-universe.dev/src/contrib/PACKAGES"
        );
        // 去掉多余的尾部斜杠
        assert_eq!(
            packages_url("https://x.dev/"),
            "https://x.dev/src/contrib/PACKAGES"
        );
    }

    /// 真·联网测试：默认被忽略（避免 CI 抖动）。手动跑：
    /// `cargo test -- --ignored fetches_real_packages`
    #[test]
    #[ignore = "需要网络"]
    fn fetches_real_packages() {
        let url = packages_url("https://jeroen.r-universe.dev");
        let body = get_text(&url).unwrap();
        assert!(body.contains("Package:"));
    }
}
