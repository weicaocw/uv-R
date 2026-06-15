//! `uvr` 命令行入口。
//!
//! 用法：
//!   uvr lock <PACKAGES 文件> <根包>...        # 从本地文件求解
//!   uvr lock --repo <仓库基址> <根包>...        # 联网抓取 PACKAGES 再求解
//! lockfile 打到标准输出。

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("lock") => lock(&args[2..]),
        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!("用法 / usage:");
    eprintln!("  uvr lock <PACKAGES-file> <root-package>...");
    eprintln!("  uvr lock --repo <repo-base-url> <root-package>...");
    ExitCode::FAILURE
}

fn lock(rest: &[String]) -> ExitCode {
    // 取得 PACKAGES 文本（联网或本地文件）+ 根包列表
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
