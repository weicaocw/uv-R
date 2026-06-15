//! `uvr` 命令行入口。
//!
//! 用法：
//!   uvr lock    <PACKAGES 文件> <根包>...
//!   uvr lock    --repo <仓库基址> <根包>...
//!   uvr install --repo <仓库基址> [--lib <目录>] <根包>...
//!
//! lock 把 lockfile 打到标准输出；install 下载并安装到**项目本地库**。

use std::path::PathBuf;
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
    eprintln!("  uvr lock    --repo <repo-base-url> <root-package>...");
    eprintln!("  uvr install --repo <repo-base-url> [--lib <dir>] <root-package>...");
    ExitCode::FAILURE
}

fn lock(rest: &[String]) -> ExitCode {
    let (text, roots) = match rest.first().map(String::as_str) {
        Some("--repo") if rest.len() >= 3 => {
            let url = uvr::fetch::packages_url(&rest[1]);
            match uvr::fetch::get_text(&url) {
                Ok(t) => (t, &rest[2..]),
                Err(e) => {
                    eprintln!("抓取失败 / fetch failed: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
        Some(_) if rest.len() >= 2 => match std::fs::read_to_string(&rest[0]) {
            Ok(t) => (t, &rest[1..]),
            Err(e) => {
                eprintln!("读不了文件 / cannot read {}: {e}", rest[0]);
                return ExitCode::FAILURE;
            }
        },
        _ => return usage(),
    };
    match uvr::commands::lock_from_packages(&text, roots) {
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

fn install(rest: &[String]) -> ExitCode {
    // 解析 --repo <url> [--lib <dir>] <pkg>...
    let mut repo: Option<String> = None;
    let mut lib = PathBuf::from("r-lib");
    let mut roots: Vec<String> = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--repo" if i + 1 < rest.len() => {
                repo = Some(rest[i + 1].clone());
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

    let Some(repo) = repo else {
        eprintln!("install 需要 --repo <url> / install needs --repo <url>");
        return usage();
    };
    if roots.is_empty() {
        eprintln!("install 需要至少一个包名 / install needs at least one package");
        return ExitCode::FAILURE;
    }

    let url = uvr::fetch::packages_url(&repo);
    let text = match uvr::fetch::get_text(&url) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("抓取失败 / fetch failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    let download_dir = PathBuf::from("target/uvr-cache");
    match uvr::commands::install_packages(&text, &roots, &repo, &lib, &download_dir) {
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
