//! 命令实现：把"读元数据 → 求解 → 渲染 lockfile / 下载安装"串起来，供命令行入口调用。
//!
//! 纯逻辑（求解、安装计划）做成可测函数；真正的网络 / 文件 / 进程 IO 在
//! `install_packages` 与 `main` 里。支持**多仓库来源**。

use crate::install;
use crate::lockfile;
use crate::metadata::PackageIndex;
use crate::resolver::{ResolveError, resolve_pubgrub};
use crate::version::Version;
use std::collections::BTreeMap;
use std::path::Path;

/// 按计划逐个下载 tarball 并用 `r_bin` 安装到 `lib_dir`；返回安装清单。
///
/// `install`（求解后）与 `sync`（按 lockfile）共用这段下载 + 安装循环。
fn run_plan(
    plan: &[InstallItem],
    lib_dir: &Path,
    download_dir: &Path,
    r_bin: &Path,
) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(download_dir).map_err(|e| e.to_string())?;
    let mut installed = Vec::new();
    for item in plan {
        let tarball = download_dir.join(format!("{}_{}.tar.gz", item.name, item.version));
        install::download(&item.url, &tarball)?;
        install::install_tarball(&tarball, lib_dir, r_bin)?;
        installed.push(format!("{} {}", item.name, item.version));
    }
    Ok(installed)
}

/// 一个元数据来源：(PACKAGES 文本, 仓库基址)。
pub type Source = (String, String);

/// 把多个来源合并成一个索引（每个包记住自己的仓库）。
fn build_index(sources: &[Source]) -> PackageIndex {
    let mut index = PackageIndex::default();
    for (text, repo) in sources {
        index.merge(PackageIndex::from_repo(text, repo));
    }
    index
}

/// 对合并后的索引求解所有根包，得到"包名 → 版本"。
fn resolve_all(
    index: &PackageIndex,
    roots: &[String],
) -> Result<BTreeMap<String, Version>, ResolveError> {
    let mut combined = BTreeMap::new();
    for root in roots {
        combined.extend(resolve_pubgrub(index, root)?);
    }
    Ok(combined)
}

/// 多源求解并渲染 lockfile 文本。
pub fn lock_from_sources(sources: &[Source], roots: &[String]) -> Result<String, ResolveError> {
    let index = build_index(sources);
    Ok(lockfile::render(&resolve_all(&index, roots)?))
}

/// 单一本地 `PACKAGES` 文本的便捷封装（来源仓库为空）。
pub fn lock_from_packages(packages_text: &str, roots: &[String]) -> Result<String, ResolveError> {
    lock_from_sources(&[(packages_text.to_string(), String::new())], roots)
}

/// 一项待安装：包名、版本、源码 tarball 的 URL（指向该包自己的仓库）。
#[derive(Debug, PartialEq, Eq)]
pub struct InstallItem {
    pub name: String,
    pub version: String,
    pub url: String,
}

/// 纯函数：从多个来源解析出"要装哪些包、各自 tarball URL"——按**每个包自己的仓库**拼地址。
pub fn install_plan(
    sources: &[Source],
    roots: &[String],
) -> Result<Vec<InstallItem>, ResolveError> {
    let index = build_index(sources);
    let resolved = resolve_all(&index, roots)?;
    Ok(resolved
        .into_iter()
        .map(|(name, version)| {
            // 找到该 name+version 的包，用它自己的仓库拼下载地址
            let repo = index
                .versions_of(&name)
                .iter()
                .find(|p| p.version == version)
                .map(|p| p.repo.as_str())
                .unwrap_or("");
            let version = version.to_string();
            let url = install::tarball_url(repo, &name, &version);
            InstallItem { name, version, url }
        })
        .collect())
}

/// 按计划下载并安装到**项目本地库** `lib_dir`；tarball 暂存到 `download_dir`。
///
/// `r_bin` 是用来跑 `R CMD INSTALL` 的 R 可执行文件（由 `rversion::resolve_r` 选出）。
pub fn install_packages(
    sources: &[Source],
    roots: &[String],
    lib_dir: &Path,
    download_dir: &Path,
    r_bin: &Path,
) -> Result<Vec<String>, String> {
    let plan = install_plan(sources, roots).map_err(|e| format!("{e:?}"))?;
    run_plan(&plan, lib_dir, download_dir, r_bin)
}

/// 纯函数：按 lockfile（已锁定的 `name → version`）在仓库索引里定位每个包、拼下载计划。
///
/// 与 `install_plan` 的关键区别：**不求解**。它严格安装 lockfile 里写死的版本，
/// 即使仓库里已有更高版本——这正是 `sync` 的意义：可复现，杜绝"求解漂移"。
/// 锁定的版本在索引里找不到（仓库变了 / 包没了）则报错并指名是哪个包。
pub fn sync_plan(
    locked: &BTreeMap<String, Version>,
    sources: &[Source],
) -> Result<Vec<InstallItem>, String> {
    let index = build_index(sources);
    locked
        .iter()
        .map(|(name, version)| {
            let found = index
                .versions_of(name)
                .iter()
                .find(|p| p.version == *version)
                .ok_or_else(|| {
                    format!(
                        "锁定的 {name} {version} 在仓库里找不到 / locked {name} {version} not found in repos"
                    )
                })?;
            let url = install::tarball_url(&found.repo, name, &version.to_string());
            Ok(InstallItem {
                name: name.clone(),
                version: version.to_string(),
                url,
            })
        })
        .collect()
}

/// 读 lockfile 文本，按其中锁定的版本下载并安装到 `lib_dir`（不求解）。
pub fn sync_from_lock(
    lockfile_text: &str,
    sources: &[Source],
    lib_dir: &Path,
    download_dir: &Path,
    r_bin: &Path,
) -> Result<Vec<String>, String> {
    let locked =
        lockfile::parse(lockfile_text).ok_or("lockfile 解析失败 / cannot parse lockfile")?;
    let plan = sync_plan(&locked, sources)?;
    run_plan(&plan, lib_dir, download_dir, r_bin)
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

    fn one_source(repo: &str) -> Vec<Source> {
        vec![(PACKAGES.to_string(), repo.to_string())]
    }

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
        let plan =
            install_plan(&one_source("https://repo.example"), &["pkgB".to_string()]).unwrap();
        let pkgb = plan.iter().find(|i| i.name == "pkgB").unwrap();
        assert_eq!(
            pkgb.url,
            "https://repo.example/src/contrib/pkgB_2.0.0.tar.gz"
        );
    }

    #[test]
    fn sync_plan_installs_exact_locked_version_no_drift() {
        // 索引里有 pkgA 1.0.0 和 1.2.0；锁定 1.0.0 → 计划必须用 1.0.0（不漂到 1.2.0）。
        let mut locked = BTreeMap::new();
        locked.insert("pkgA".to_string(), Version::parse("1.0.0").unwrap());
        let plan = sync_plan(&locked, &one_source("https://repo.example")).unwrap();
        let a = plan.iter().find(|i| i.name == "pkgA").unwrap();
        assert_eq!(a.version, "1.0.0");
        assert_eq!(a.url, "https://repo.example/src/contrib/pkgA_1.0.0.tar.gz");
    }

    #[test]
    fn sync_plan_errors_on_unknown_locked_version() {
        let mut locked = BTreeMap::new();
        locked.insert("pkgA".to_string(), Version::parse("9.9.9").unwrap());
        let err = sync_plan(&locked, &one_source("https://repo.example")).unwrap_err();
        assert!(err.contains("pkgA")); // 报错指名是哪个包
    }

    #[test]
    fn sync_from_lock_rejects_bad_lockfile() {
        let bad = "not a valid lockfile line";
        let err = sync_from_lock(
            bad,
            &one_source("https://r"),
            Path::new("x"),
            Path::new("y"),
            Path::new("R"),
        )
        .unwrap_err();
        assert!(err.contains("lockfile"));
    }

    #[test]
    fn install_plan_uses_each_packages_own_repo() {
        // pkgX 在 r1、依赖 pkgY；pkgY 在 r2。各自的下载地址应指向各自的仓库。
        let sources = vec![
            (
                "Package: pkgX\nVersion: 1.0\nImports: pkgY\n".to_string(),
                "https://r1".to_string(),
            ),
            (
                "Package: pkgY\nVersion: 2.0\n".to_string(),
                "https://r2".to_string(),
            ),
        ];
        let plan = install_plan(&sources, &["pkgX".to_string()]).unwrap();
        let x = plan.iter().find(|i| i.name == "pkgX").unwrap();
        let y = plan.iter().find(|i| i.name == "pkgY").unwrap();
        assert_eq!(x.url, "https://r1/src/contrib/pkgX_1.0.tar.gz");
        assert_eq!(y.url, "https://r2/src/contrib/pkgY_2.0.tar.gz");
    }
}
