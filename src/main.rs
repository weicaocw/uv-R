//! `uvr` 命令行入口。
//!
//! 用法：
//!   uvr lock    <PACKAGES 文件> <根包>...
//!   uvr lock    --repo <仓库> [--repo ...] <根包>...
//!   uvr install --repo <仓库> [--repo ...] [--lib <目录>] [--jobs <N>] <根包>...
//!   uvr sync    --repo <仓库> [--repo ...] [--lib <目录>] [--jobs <N>] [<lockfile>]
//!   uvr r list | which | pin [<版本>] | install <版本>
//!
//! 多仓库：抓取并合并多个仓库后求解 / 安装。元数据与 tarball 都走 `.uvr-cache/` 暖缓存。
//! tarball **并行下载**（`--jobs`，默认 = CPU 核数），安装串行。
//! `uvr sync`：按 lockfile 还原（不求解）。`uvr r ...`：管理本机 R 版本。

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("lock") => lock(&args[2..]),
        Some("install") => install(&args[2..]),
        Some("sync") => sync(&args[2..]),
        Some("r") => r_command(&args[2..]),
        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!("用法 / usage:");
    eprintln!("  uvr lock    <PACKAGES-file> <root-package>...");
    eprintln!("  uvr lock    --repo <url> [--repo <url2> ...] <root-package>...");
    eprintln!(
        "  uvr install --repo <url> [--repo <url2> ...] [--lib <dir>] [--jobs <N>] <root-package>..."
    );
    eprintln!(
        "  uvr sync    --repo <url> [--repo <url2> ...] [--lib <dir>] [--jobs <N>] [<lockfile>]"
    );
    eprintln!(
        "  uvr r list | which | pin [<version>] | install <version>   # 管理 R 版本 / manage R versions"
    );
    ExitCode::FAILURE
}

/// 解析当前项目该用的 R（pin > 最高），打印并返回其路径；失败则打印错误、返回 None。
/// `install` 与 `sync` 共用，保证两者"用哪个 R"的行为一致。
fn resolve_r_bin() -> Option<PathBuf> {
    match uvr::rversion::resolve_r(Path::new(".")) {
        Ok(r) => {
            eprintln!("→ 使用 R / using R {}: {}", r.version, r.path.display());
            Some(r.path)
        }
        Err(e) => {
            eprintln!("{e}");
            None
        }
    }
}

/// `uvr r <子命令>`：管理本机 R 版本。
fn r_command(rest: &[String]) -> ExitCode {
    match rest.first().map(String::as_str) {
        Some("list") => r_list(),
        Some("which") => r_which(),
        Some("pin") => r_pin(&rest[1..]),
        Some("install") => r_install(&rest[1..]),
        _ => {
            eprintln!("用法 / usage: uvr r list | which | pin [<version>] | install <version>");
            ExitCode::FAILURE
        }
    }
}

/// `uvr r install <版本>`：uvr 不做系统级安装，委托 `rig` 或给出清晰指引（资源墙交接）。
fn r_install(args: &[String]) -> ExitCode {
    let Some(spec) = args.first() else {
        eprintln!("用法 / usage: uvr r install <version>");
        return ExitCode::FAILURE;
    };
    let present = uvr::rversion::rig_available();
    println!("{}", uvr::rversion::rig_install_hint(spec, present));
    // 没有实际安装，返回失败让脚本能据此判断（诚实：这一步把活交还给了你 / rig）。
    ExitCode::FAILURE
}

/// `uvr r list`：列出发现到的所有 R，并用 `*` 标出当前会选中的那个。
fn r_list() -> ExitCode {
    let installs = uvr::rversion::discover();
    if installs.is_empty() {
        eprintln!("未发现任何 R / no R found");
        return ExitCode::FAILURE;
    }
    let selected = uvr::rversion::resolve_r(Path::new(".")).ok();
    for i in &installs {
        let here = selected.as_ref().is_some_and(|s| s.version == i.version);
        let mark = if here { "*" } else { " " };
        println!("{mark} {} {}", i.version, i.path.display());
    }
    ExitCode::SUCCESS
}

/// `uvr r which`：打印当前项目会用的那个 R（按 pin > 最高 解析）。
fn r_which() -> ExitCode {
    match uvr::rversion::resolve_r(Path::new(".")) {
        Ok(r) => {
            println!("{} {}", r.version, r.path.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// `uvr r pin [<版本>]`：把版本写进 `./.R-version`；省略版本时钉当前解析到的 R。
fn r_pin(args: &[String]) -> ExitCode {
    let dir = Path::new(".");
    let spec = match args.first() {
        Some(s) => s.clone(),
        None => match uvr::rversion::resolve_r(dir) {
            Ok(r) => r.version.to_string(),
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        },
    };
    match uvr::rversion::write_pin(dir, &spec) {
        Ok(()) => {
            println!("已钉定 / pinned R {spec} → {}", uvr::rversion::PIN_FILE);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("写入失败 / write failed: {e}");
            ExitCode::FAILURE
        }
    }
}

/// 元数据缓存目录（暖缓存）。
fn meta_cache_dir() -> PathBuf {
    PathBuf::from(".uvr-cache/meta")
}

/// 默认并行下载并发度 = CPU 核数（取不到则退回 4）。
fn default_jobs() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// 解析 `--repo`（可多个）、`--lib`、`--jobs <N>`，其余作为位置参数（包名 / lockfile 路径）。
fn parse_flags(rest: &[String]) -> (Vec<String>, PathBuf, usize, Vec<String>) {
    let mut repos = Vec::new();
    let mut lib = PathBuf::from("r-lib");
    let mut jobs = default_jobs();
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
            "--jobs" if i + 1 < rest.len() => {
                // 非法 / 0 都退回默认值，并保证至少 1。
                jobs = rest[i + 1].parse().unwrap_or(jobs).max(1);
                i += 2;
            }
            other => {
                roots.push(other.to_string());
                i += 1;
            }
        }
    }
    (repos, lib, jobs, roots)
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
        let (repos, _lib, _jobs, roots) = parse_flags(rest);
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
    let (repos, lib, jobs, roots) = parse_flags(rest);
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
    // 用按 pin > 最高 解析出的那个 R 来跑 R CMD INSTALL；钉了却没装就明确报错、不偷偷换 R。
    let Some(r_bin) = resolve_r_bin() else {
        return ExitCode::FAILURE;
    };
    let download_dir = PathBuf::from(".uvr-cache/tarballs");
    match uvr::commands::install_packages(&sources, &roots, &lib, &download_dir, &r_bin, jobs) {
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

/// `uvr sync --repo <url>... [--lib <dir>] [<lockfile>]`：按 lockfile 还原环境（不求解）。
/// 省略 lockfile 路径时默认读 `uvr.lock`。
fn sync(rest: &[String]) -> ExitCode {
    let (repos, lib, jobs, positional) = parse_flags(rest);
    if repos.is_empty() {
        eprintln!("sync 需要 --repo <url>（下载来源）/ sync needs --repo <url>");
        return usage();
    }
    let lockpath = positional.first().map(String::as_str).unwrap_or("uvr.lock");
    let text = match std::fs::read_to_string(lockpath) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("读不了 lockfile / cannot read {lockpath}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let sources = match fetch_sources(&repos, &meta_cache_dir()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("抓取失败 / fetch failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(r_bin) = resolve_r_bin() else {
        return ExitCode::FAILURE;
    };
    let download_dir = PathBuf::from(".uvr-cache/tarballs");
    match uvr::commands::sync_from_lock(&text, &sources, &lib, &download_dir, &r_bin, jobs) {
        Ok(installed) => {
            for p in &installed {
                println!("synced {p}");
            }
            eprintln!(
                "→ 已按 {lockpath} 还原到项目本地库 / restored from {lockpath} into: {}",
                lib.display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("同步失败 / sync failed: {e}");
            ExitCode::FAILURE
        }
    }
}
