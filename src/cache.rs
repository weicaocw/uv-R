//! 本地缓存：把抓到的文本（如 `PACKAGES`）按 URL 存到缓存目录，重复使用走"暖缓存"。
//!
//! 缓存只是优化——写失败时静默忽略，绝不让主流程因缓存出错而失败。

use std::path::{Path, PathBuf};

/// 由缓存目录与 URL 生成确定性的缓存文件路径（URL 中非字母数字字符转为下划线）。
pub fn cache_path(cache_dir: &Path, url: &str) -> PathBuf {
    let key: String = url
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    cache_dir.join(key)
}

/// 读缓存：命中返回内容，未命中（或读失败）返回 None。
pub fn read(cache_dir: &Path, url: &str) -> Option<String> {
    std::fs::read_to_string(cache_path(cache_dir, url)).ok()
}

/// 写缓存：失败静默忽略（缓存写不进去不影响正确性）。
pub fn write(cache_dir: &Path, url: &str, content: &str) {
    let _ = std::fs::create_dir_all(cache_dir);
    let _ = std::fs::write(cache_path(cache_dir, url), content);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_path_is_deterministic_and_safe() {
        let dir = Path::new("/tmp/uvr-cache");
        let p1 = cache_path(dir, "https://x.dev/src/contrib/PACKAGES");
        let p2 = cache_path(dir, "https://x.dev/src/contrib/PACKAGES");
        assert_eq!(p1, p2);
        let name = p1.file_name().unwrap().to_str().unwrap();
        assert!(name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
    }

    #[test]
    fn write_then_read_round_trips() {
        let dir = std::env::temp_dir().join("uvr_cache_unit_test");
        let _ = std::fs::remove_dir_all(&dir);
        write(&dir, "u://k", "hello");
        assert_eq!(read(&dir, "u://k").as_deref(), Some("hello"));
        assert_eq!(read(&dir, "u://missing"), None);
    }
}
