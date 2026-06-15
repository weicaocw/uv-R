//! `uvr` 命令行入口。
//!
//! 用法：
//!   uvr lock    <PACKAGES 文件> <根包>...
//!   uvr lock    --repo <仓库> [--repo <仓库2> ...] <根包>...
//!   uvr install --repo <仓库> [--repo <仓库2> ...] [--lib <目录>] <根包>...
//!
//! 多仓库：抓取并合并多个仓库后求解 / 安装。元数据与 tarball 都走 `.uvr-cache/` 暖缓存。

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("lock") => lock(&args[2..]),
        Some("install") => install(&args[2..]),
        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!("用法 / usage:");
    eprintln!("  uvr lock    <PACKAGES-file> <root-package>...");
    eprintln!("  uvr lock    --repo <url> [--repo <url2> ...] <root-package>...");
    eprintln!("  uvr install --repo <url> [--repo <url2> ...] [--lib <dir>] <root-package>...");
    ExitCode::FAILURE
}

/// 元数据缓存目录（暖缓存）。
fn meta_cache_dir() -> PathBuf {
    PathBuf::from(".uvr-cache/meta")
}

/// 解析 `--repo`（可多个）、`--lib`，其余作为根包名。
fn parse_flags(rest: &[String]) -> (Vec<String>, PathBuf, Vec<String>) {
    let mut repos = Vec::new();
    let mut lib = PathBuf::from("r-lib");
    let mut roots = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--repo" if i + 1 < rest.len() => {
                repos.push(rest[i + 1].clone());
                i += 2;
            }
            "--lib" if i + 1 < rest.len() => {
                lib = PathBuf::from(&rest[i + 1]);
                i += 2;
            }
            other => {
                roots.push(other.to_string());
                i += 1;
            }
        }
    }
    (repos, lib, roots)
}

/// 抓取每个仓库的 PACKAGES（走缓存），组成 (文本, 仓库) 源列表。
fn fetch_sources(repos: &[String], cache_dir: &Path) -> Result<Vec<(String, String)>, String> {
    let mut sources = Vec::new();
    for repo in repos {
        let url = uvr::fetch::packages_url(repo);
        let text =
            uvr::fetch::get_text_cached(&url, cache_dir).map_err(|e| format!("{repo}: {e}"))?;
        sources.push((text, repo.clone()));
    }
    Ok(sources)
}

fn finish_lock(sources: &[(String, String)], roots: &[String]) -> ExitCode {
    match uvr::commands::lock_from_sources(sources, roots) {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("求解失败 / resolve failed: {e:?}");
            ExitCode::FAILURE
        }
    }
}

fn lock(rest: &[String]) -> ExitCode {
    if rest.iter().any(|a| a == "--repo") {
        let (repos, _lib, roots) = parse_flags(rest);
        match fetch_sources(&repos, &meta_cache_dir()) {
            Ok(sources) => finish_lock(&sources, &roots),
            Err(e) => {
                eprintln!("抓取失败 / fetch failed: {e}");
                ExitCode::FAILURE
            }
        }
    } else if rest.len() >= 2 {
        // 本地文件：uvr lock <file> <pkg>...
        match std::fs::read_to_string(&rest[0]) {
            Ok(text) => finish_lock(&[(text, String::new())], &rest[1..]),
            Err(e) => {
                eprintln!("读不了文件 / cannot read {}: {e}", rest[0]);
                ExitCode::FAILURE
            }
        }
    } else {
        usage()
    }
}

fn install(rest: &[String]) -> ExitCode {
    let (repos, lib, roots) = parse_flags(rest);
    if repos.is_empty() {
        eprintln!("install 需要 --repo <url> / install needs --repo <url>");
        return usage();
    }
    if roots.is_empty() {
        eprintln!("install 需要至少一个包名 / install needs at least one package");
        return ExitCode::FAILURE;
    }
    let sources = match fetch_sources(&repos, &meta_cache_dir()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("抓取失败 / fetch failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    let download_dir = PathBuf::from(".uvr-cache/tarballs");
    match uvr::commands::install_packages(&sources, &roots, &lib, &download_dir) {
        Ok(installed) => {
            for p in &installed {
                println!("installed {p}");
            }
            eprintln!(
                "→ 已安装到项目本地库 / installed into project-local lib: {}",
                lib.display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("安装失败 / install failed: {e}");
            ExitCode::FAILURE
        }
    }
}
