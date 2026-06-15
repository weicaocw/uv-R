//! R 版本管理：发现系统里的 R、项目级钉版本（`.R-version`）、按优先级选择该用哪个 R。
//!
//! 对标 uv 的 `uv python` 家族：uvr 不自己编译 R（就像 uv 不自己编译 Python），
//! 只负责**发现 / 选择 / 使用**已安装的 R；**获取**一个新 R 是系统级操作，交给 `rig` 或交还用户。

use crate::version::Version;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 一处已安装的 R：它的可执行文件路径，以及解析出来的版本。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RInstall {
    pub path: PathBuf,
    pub version: Version,
}

/// 不依赖 `PATH` 的、常见的 R 安装位置（macOS + Linux）。
const KNOWN_R_PATHS: &[&str] = &[
    "/usr/local/bin/R",    // Homebrew(Intel) / CRAN 默认软链
    "/opt/homebrew/bin/R", // Homebrew(Apple Silicon)
    "/Library/Frameworks/R.framework/Resources/bin/R", // CRAN .pkg 当前版
    "/usr/bin/R",          // Linux 发行版
    "/usr/lib/R/bin/R",    // Linux（部分发行版）
];

/// 去重但**保持原顺序**（先出现的优先）。
fn dedup_keep_order(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = std::collections::HashSet::new();
    paths
        .into_iter()
        .filter(|p| seen.insert(p.clone()))
        .collect()
}

/// 纯函数：给定一段 `PATH` 文本，列出所有**候选**的 R 可执行文件路径。
///
/// = `PATH` 里每个目录下的 `R` + 一组已知安装位置，去重保序。纯逻辑、不碰文件系统，便于单测。
pub fn r_candidate_paths_from(path_var: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in path_var.split(':') {
        if !dir.is_empty() {
            out.push(Path::new(dir).join("R"));
        }
    }
    for known in KNOWN_R_PATHS {
        out.push(PathBuf::from(known));
    }
    dedup_keep_order(out)
}

/// 纯函数：按**版本**去重（同一版本只留先出现的那个，即候选顺序里优先的路径）。
pub fn dedup_installs(installs: Vec<RInstall>) -> Vec<RInstall> {
    let mut seen = std::collections::HashSet::new();
    installs
        .into_iter()
        .filter(|i| seen.insert(i.version.clone()))
        .collect()
}

/// 探测单个路径：运行 `<path> --version`，解析出 `RInstall`；跑不起来 / 认不出则 `None`。
///
/// 有副作用（起进程），所以不在普通单测里跑；靠下面 `#[ignore]` 的真·探测覆盖。
pub fn probe_r(path: &Path) -> Option<RInstall> {
    let output = Command::new(path).arg("--version").output().ok()?;
    // `R --version` 打到 stdout；个别构建打到 stderr，两处都试。
    let version = parse_r_version(&String::from_utf8_lossy(&output.stdout))
        .or_else(|| parse_r_version(&String::from_utf8_lossy(&output.stderr)))?;
    Some(RInstall {
        path: path.to_path_buf(),
        version,
    })
}

/// 发现本机所有的 R：候选路径逐个探测、再按版本去重。
pub fn discover() -> Vec<RInstall> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    let installs = r_candidate_paths_from(&path_var)
        .iter()
        .filter_map(|p| probe_r(p))
        .collect();
    dedup_installs(installs)
}

/// 从 `R --version` 的输出里解析出版本号。
///
/// 典型首行：`R version 4.5.2 (2025-10-31) -- "..."`。
/// 取 `version` 之后紧跟的那个词（`4.5.2`），而不是括号里的日期（`2025-10-31`）。
pub fn parse_r_version(output: &str) -> Option<Version> {
    let mut tokens = output.split_whitespace();
    while let Some(tok) = tokens.next() {
        if tok == "version" {
            let candidate = tokens.next()?.trim_end_matches([',', ')']);
            return Version::parse(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_first_line() {
        let line = r#"R version 4.5.2 (2025-10-31) -- "[Not] Part in a Rumble""#;
        assert_eq!(parse_r_version(line), Version::parse("4.5.2"));
    }

    #[test]
    fn takes_version_not_the_date() {
        // 括号里的日期含 '-'，若误取就会变成 2025.10.31——确保我们取的是 4.5.2。
        let v = parse_r_version("R version 4.5.2 (2025-10-31) -- x").unwrap();
        assert_eq!(v, Version::parse("4.5.2").unwrap());
        assert_ne!(v, Version::parse("2025.10.31").unwrap());
    }

    #[test]
    fn parses_bare_version() {
        assert_eq!(parse_r_version("R version 3.6.1"), Version::parse("3.6.1"));
    }

    #[test]
    fn none_when_no_version_word() {
        assert_eq!(parse_r_version("R Under development (unstable)"), None);
        assert_eq!(parse_r_version("totally unrelated text"), None);
    }

    #[test]
    fn candidates_cover_path_and_known_roots() {
        let c = r_candidate_paths_from("/x/bin:/y/bin");
        assert!(c.contains(&PathBuf::from("/x/bin/R")));
        assert!(c.contains(&PathBuf::from("/y/bin/R")));
        assert!(c.contains(&PathBuf::from("/usr/local/bin/R"))); // 已知位置也在
    }

    #[test]
    fn candidates_dedup_path_vs_known() {
        // /usr/local/bin 同时来自 PATH 和已知位置，去重后只出现一次。
        let c = r_candidate_paths_from("/usr/local/bin");
        let n = c
            .iter()
            .filter(|p| *p == &PathBuf::from("/usr/local/bin/R"))
            .count();
        assert_eq!(n, 1);
    }

    #[test]
    fn candidates_skip_empty_path_segments() {
        // PATH 里的空段（如 "a::b" 中间的空目录）应被跳过，不生成裸 "R"。
        let c = r_candidate_paths_from("/a::/b");
        assert!(!c.contains(&PathBuf::from("R")));
    }

    fn install(path: &str, ver: &str) -> RInstall {
        RInstall {
            path: PathBuf::from(path),
            version: Version::parse(ver).unwrap(),
        }
    }

    #[test]
    fn dedup_installs_collapses_same_version() {
        let got = dedup_installs(vec![
            install("/usr/local/bin/R", "4.5.2"),
            install("/opt/homebrew/bin/R", "4.5.2"), // 同版本：丢弃（保留先出现的）
            install("/usr/bin/R", "4.4.0"),
        ]);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].path, PathBuf::from("/usr/local/bin/R"));
        assert_eq!(got[1].version, Version::parse("4.4.0").unwrap());
    }

    /// 真·发现本机的 R：默认忽略（需要真实的 R）。手动：
    /// `cargo test -- --ignored discovers_at_least_one_r`
    #[test]
    #[ignore = "需要本机安装 R"]
    fn discovers_at_least_one_r() {
        let found = discover();
        assert!(!found.is_empty(), "应至少发现一个 R");
    }
}
