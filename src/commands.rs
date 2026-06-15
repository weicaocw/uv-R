//! 命令实现：把"读元数据 → 求解 → 渲染 lockfile / 下载安装"串起来，供命令行入口调用。
//!
//! 纯逻辑（求解、安装计划）做成可测函数；真正的网络 / 文件 / 进程 IO 在
//! `install_packages` 与 `main` 里。

use crate::install;
use crate::lockfile;
use crate::metadata::PackageIndex;
use crate::resolver::{ResolveError, resolve_pubgrub};
use crate::version::Version;
use std::collections::BTreeMap;
use std::path::Path;

/// 给定 `PACKAGES` 文本与若干根包，解析依赖并渲染成 lockfile 文本。
pub fn lock_from_packages(packages_text: &str, roots: &[String]) -> Result<String, ResolveError> {
    let index = PackageIndex::from_packages_file(packages_text);
    let mut combined: BTreeMap<String, Version> = BTreeMap::new();
    for root in roots {
        combined.extend(resolve_pubgrub(&index, root)?);
    }
    Ok(lockfile::render(&combined))
}

/// 一项待安装：包名、版本、源码 tarball 的 URL。
#[derive(Debug, PartialEq, Eq)]
pub struct InstallItem {
    pub name: String,
    pub version: String,
    pub url: String,
}

/// 纯函数：解析出"要装哪些包、各自 tarball URL"——不触网、不安装，便于单测。
pub fn install_plan(
    packages_text: &str,
    roots: &[String],
    repo_base: &str,
) -> Result<Vec<InstallItem>, ResolveError> {
    let index = PackageIndex::from_packages_file(packages_text);
    let mut combined: BTreeMap<String, Version> = BTreeMap::new();
    for root in roots {
        combined.extend(resolve_pubgrub(&index, root)?);
    }
    Ok(combined
        .into_iter()
        .map(|(name, version)| {
            let version = version.to_string();
            let url = install::tarball_url(repo_base, &name, &version);
            InstallItem { name, version, url }
        })
        .collect())
}

/// 按计划下载并安装到**项目本地库** `lib_dir`；tarball 暂存到 `download_dir`。
/// 返回已安装的 "name version" 列表。
pub fn install_packages(
    packages_text: &str,
    roots: &[String],
    repo_base: &str,
    lib_dir: &Path,
    download_dir: &Path,
) -> Result<Vec<String>, String> {
    let plan = install_plan(packages_text, roots, repo_base).map_err(|e| format!("{e:?}"))?;
    std::fs::create_dir_all(download_dir).map_err(|e| e.to_string())?;
    let mut installed = Vec::new();
    for item in &plan {
        let tarball = download_dir.join(format!("{}_{}.tar.gz", item.name, item.version));
        install::download(&item.url, &tarball)?;
        install::install_tarball(&tarball, lib_dir)?;
        installed.push(format!("{} {}", item.name, item.version));
    }
    Ok(installed)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PACKAGES: &str = "\
Package: pkgA
Version: 1.0.0

Package: pkgA
Version: 1.2.0

Package: pkgB
Version: 2.0.0
Depends: R (>= 3.0.0), pkgA (>= 1.1.0)
";

    #[test]
    fn locks_package_and_deps() {
        let text = lock_from_packages(PACKAGES, &["pkgB".to_string()]).unwrap();
        assert!(text.contains("pkgB 2.0.0"));
        assert!(text.contains("pkgA 1.2.0"));
    }

    #[test]
    fn propagates_resolve_error() {
        let pkgs = "Package: x\nVersion: 1.0\nDepends: nope (>= 1.0)\n";
        assert!(lock_from_packages(pkgs, &["x".to_string()]).is_err());
    }

    #[test]
    fn builds_install_plan_with_urls() {
        let plan = install_plan(PACKAGES, &["pkgB".to_string()], "https://repo.example").unwrap();
        let names: Vec<&str> = plan.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"pkgA"));
        assert!(names.contains(&"pkgB"));
        let pkgb = plan.iter().find(|i| i.name == "pkgB").unwrap();
        assert_eq!(
            pkgb.url,
            "https://repo.example/src/contrib/pkgB_2.0.0.tar.gz"
        );
    }
}
