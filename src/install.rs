//! 下载并安装 R 包到**项目本地**库目录（绝不碰全局 / 用户级 R 环境）。
//!
//! 安装的实际构建交给 R 自己的 `R CMD INSTALL`（就像 uv 不自己实现 Python 编译一样）；
//! 我们只负责"下载 tarball + 调用 R + 指定本地库目录"。

use std::path::Path;
use std::process::Command;

/// 源码 tarball 的 URL：`<base>/src/contrib/<name>_<version>.tar.gz`。
pub fn tarball_url(repo_base: &str, name: &str, version: &str) -> String {
    format!(
        "{}/src/contrib/{}_{}.tar.gz",
        repo_base.trim_end_matches('/'),
        name,
        version
    )
}

/// 下载 URL 到本地文件（二进制流式写入，省内存）。
pub fn download(url: &str, dest: &Path) -> Result<(), String> {
    if dest.exists() {
        return Ok(()); // 已缓存的 tarball，跳过下载
    }
    let resp = ureq::get(url).call().map_err(|e| e.to_string())?;
    let mut reader = resp.into_reader();
    let mut file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    std::io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;
    Ok(())
}

/// 把字节切片渲染成小写十六进制串。
fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// 校验文件内容的校验和是否与 `expected` 相符。
///
/// `expected` 形如 `sha256:<hex>`（r-universe / PPM）或 `md5:<hex>`（CRAN）。按算法前缀分派计算；
/// 为空、无前缀或未知算法则**跳过**校验、放行（不假装能验证拿不到的东西）。
/// 不符则返回错误，调用方据此**拒绝安装**——防传输损坏 / 篡改。
pub fn verify_hash(file: &Path, expected: &str) -> Result<(), String> {
    use md5::Md5;
    use sha2::{Digest, Sha256};
    let Some((algo, want)) = expected.split_once(':') else {
        return Ok(()); // 空 / 无前缀：跳过
    };
    let bytes = std::fs::read(file).map_err(|e| e.to_string())?;
    let got = match algo {
        "sha256" => to_hex(&Sha256::digest(&bytes)),
        "md5" => to_hex(&Md5::digest(&bytes)),
        _ => return Ok(()), // 未知算法：跳过
    };
    if got.eq_ignore_ascii_case(want) {
        Ok(())
    } else {
        Err(format!(
            "{algo} 校验和不符 / checksum mismatch: 期望/expected {want}, 实得/got {got}"
        ))
    }
}

/// 用 `<r_bin> CMD INSTALL -l <lib_dir>` 把 tarball 装进**指定的项目本地库**。
///
/// `-l <lib_dir>` 把安装目标限定在我们给的目录，因此不会污染系统 / 用户级 R 库。
/// `r_bin` 是要用的 R 可执行文件（由 `rversion::resolve_r` 选出，对标"用钉定的解释器装包"）。
pub fn install_tarball(tarball: &Path, lib_dir: &Path, r_bin: &Path) -> Result<(), String> {
    std::fs::create_dir_all(lib_dir).map_err(|e| e.to_string())?;
    let status = Command::new(r_bin)
        .arg("CMD")
        .arg("INSTALL")
        .arg("-l")
        .arg(lib_dir)
        .arg(tarball)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("R CMD INSTALL 退出码 {:?}", status.code()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_handles_sha256_md5_skip_and_mismatch() {
        use md5::Md5;
        use sha2::{Digest, Sha256};
        let dir = std::path::PathBuf::from("target/test-verify");
        let _ = std::fs::create_dir_all(&dir);
        let f = dir.join("blob");
        let content = b"uvr integrity test";
        std::fs::write(&f, content).unwrap();
        let sha: String = Sha256::digest(content)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let md5: String = Md5::digest(content)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert!(verify_hash(&f, &format!("sha256:{sha}")).is_ok()); // sha256 正确
        assert!(verify_hash(&f, &format!("md5:{md5}")).is_ok()); // md5 正确
        assert!(verify_hash(&f, "sha256:0000").is_err()); // sha256 错误 → 拒绝
        assert!(verify_hash(&f, "md5:0000").is_err()); // md5 错误 → 拒绝
        assert!(verify_hash(&f, "").is_ok()); // 空 → 跳过
        assert!(verify_hash(&f, "crc32:abcd").is_ok()); // 未知算法 → 跳过
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn builds_tarball_url() {
        assert_eq!(
            tarball_url("https://cran.r-project.org/", "praise", "1.0.0"),
            "https://cran.r-project.org/src/contrib/praise_1.0.0.tar.gz"
        );
    }

    /// 真·下载 + 安装到项目本地库：默认忽略（需网络与 R）。手动：
    /// `cargo test -- --ignored installs_praise_locally`
    #[test]
    #[ignore = "需要网络与 R"]
    fn installs_praise_locally() {
        let lib = std::path::PathBuf::from("target/test-r-lib");
        let tarball = std::path::PathBuf::from("target/praise_1.0.0.tar.gz");
        let url = tarball_url("https://cran.r-project.org", "praise", "1.0.0");
        download(&url, &tarball).unwrap();
        install_tarball(&tarball, &lib, std::path::Path::new("R")).unwrap();
        // 装好后，本地库里应出现 praise/DESCRIPTION
        assert!(lib.join("praise").join("DESCRIPTION").exists());
    }
}
