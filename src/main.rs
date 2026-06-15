//! `uvr` 命令行入口。
//!
//! 当前支持：`uvr lock <PACKAGES 文件> <根包>...`
//! —— 读取 PACKAGES、求解依赖、把 lockfile 打到标准输出。

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("lock") if args.len() >= 4 => lock(&args[2], &args[3..]),
        _ => {
            eprintln!("用法 / usage: uvr lock <PACKAGES-file> <root-package>...");
            ExitCode::FAILURE
        }
    }
}

fn lock(file: &str, roots: &[String]) -> ExitCode {
    let text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("读不了文件 / cannot read {file}: {e}");
            return ExitCode::FAILURE;
        }
    };
    match uvr::commands::lock_from_packages(&text, roots) {
        Ok(lock) => {
            print!("{lock}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("求解失败 / resolve failed: {e:?}");
            ExitCode::FAILURE
        }
    }
}
