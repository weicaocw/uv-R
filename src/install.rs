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
