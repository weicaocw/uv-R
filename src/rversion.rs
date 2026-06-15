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

/// 项目级钉版本文件名，对标 uv 的 `.python-version`。
pub const PIN_FILE: &str = ".R-version";

/// 某目录下钉版本文件的完整路径。
pub fn pin_path(dir: &Path) -> PathBuf {
    dir.join(PIN_FILE)
}

/// 纯函数：渲染 `.R-version` 内容（版本规格 + 换行）。
pub fn render_pin(spec: &str) -> String {
    format!("{}\n", spec.trim())
}

/// 纯函数：解析 `.R-version`——取第一行非空、非注释（`#`）的内容并 trim。
pub fn parse_pin(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
}

/// 读 `dir/.R-version`；不存在 / 全是空行注释则 `None`。
pub fn read_pin(dir: &Path) -> Option<String> {
    parse_pin(&std::fs::read_to_string(pin_path(dir)).ok()?)
}

/// 把版本规格写进 `dir/.R-version`。
pub fn write_pin(dir: &Path, spec: &str) -> std::io::Result<()> {
    std::fs::write(pin_path(dir), render_pin(spec))
}

/// 纯函数：版本规格 `spec` 是否匹配版本 `v`——按点分段做**前缀**匹配。
///
/// `4.5` 匹配 `4.5.2`；`4` 匹配任意 `4.x`；`4.5.2` 只精确匹配 `4.5.2`；
/// `4.5.2` 不匹配 `4.5`（规格比实际更具体时不算命中）。
pub fn version_matches(spec: &str, v: &Version) -> bool {
    let full = v.to_string();
    let want: Vec<&str> = spec.split('.').collect();
    let have: Vec<&str> = full.split('.').collect();
    want.len() <= have.len() && want.iter().zip(&have).all(|(a, b)| a == b)
}

/// 选 R 时可能的失败。
#[derive(Debug, PartialEq, Eq)]
pub enum RSelectError {
    /// 一个 R 都没发现。
    NoRInstalled,
    /// 钉了某版本，但没有任何已装的 R 匹配它。
    PinnedNotFound(String),
}

impl std::fmt::Display for RSelectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RSelectError::NoRInstalled => {
                write!(f, "未发现任何 R / no R installation found")
            }
            RSelectError::PinnedNotFound(spec) => {
                write!(
                    f,
                    "钉定的 R 版本 {spec} 未安装 / pinned R version {spec} is not installed"
                )
            }
        }
    }
}

/// 纯函数：按优先级选出该用的那个 R。
///
/// 规则：有 pin → 在已装里挑**匹配且最高**的；无 pin → 挑**最高版本**。
/// 一个都没装 → `NoRInstalled`；钉了却没装 → `PinnedNotFound`。
pub fn select_r(pin: Option<&str>, installs: &[RInstall]) -> Result<RInstall, RSelectError> {
    if installs.is_empty() {
        return Err(RSelectError::NoRInstalled);
    }
    match pin {
        Some(spec) => installs
            .iter()
            .filter(|i| version_matches(spec, &i.version))
            .max_by(|a, b| a.version.cmp(&b.version))
            .cloned()
            .ok_or_else(|| RSelectError::PinnedNotFound(spec.to_string())),
        None => Ok(installs
            .iter()
            .max_by(|a, b| a.version.cmp(&b.version))
            .cloned()
            .expect("已确认非空")),
    }
}

/// 把"读 pin + 发现 + 选择"串起来：项目目录 → 最终该用的 R。
pub fn resolve_r(project_dir: &Path) -> Result<RInstall, RSelectError> {
    let pin = read_pin(project_dir);
    select_r(pin.as_deref(), &discover())
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

    #[test]
    fn pin_render_parse_round_trip() {
        assert_eq!(parse_pin(&render_pin("4.5.2")), Some("4.5.2".to_string()));
    }

    #[test]
    fn pin_skips_comments_and_blanks() {
        assert_eq!(
            parse_pin("# 注释\n\n  4.4.0  \n"),
            Some("4.4.0".to_string())
        );
    }

    #[test]
    fn pin_none_when_only_comments() {
        assert_eq!(parse_pin("# 只有注释\n\n"), None);
    }

    #[test]
    fn pin_round_trips_on_disk() {
        let dir = PathBuf::from("target/test-rversion-pin");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_pin(&dir, "4.5.2").unwrap();
        assert_eq!(read_pin(&dir), Some("4.5.2".to_string()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn version_matches_prefix() {
        let v = Version::parse("4.5.2").unwrap();
        assert!(version_matches("4.5.2", &v)); // 精确
        assert!(version_matches("4.5", &v)); // 前缀
        assert!(version_matches("4", &v)); // 任意 4.x
        assert!(!version_matches("4.6", &v)); // 不同小版本
        assert!(!version_matches("4.5.2.1", &v)); // 规格更具体 → 不匹配
    }

    #[test]
    fn select_errors_when_no_r() {
        assert_eq!(select_r(None, &[]), Err(RSelectError::NoRInstalled));
    }

    #[test]
    fn select_without_pin_takes_highest() {
        let installs = vec![install("/a/R", "4.4.0"), install("/b/R", "4.5.2")];
        assert_eq!(select_r(None, &installs).unwrap().version, ver("4.5.2"));
    }

    #[test]
    fn select_with_pin_picks_matching_highest() {
        let installs = vec![
            install("/a/R", "4.4.0"),
            install("/b/R", "4.4.2"),
            install("/c/R", "4.5.2"),
        ];
        // pin "4.4" 匹配 4.4.0 与 4.4.2，取更高的 4.4.2（而非全局最高 4.5.2）。
        assert_eq!(
            select_r(Some("4.4"), &installs).unwrap().version,
            ver("4.4.2")
        );
    }

    #[test]
    fn select_with_unmatched_pin_errors() {
        let installs = vec![install("/a/R", "4.5.2")];
        assert_eq!(
            select_r(Some("3.6"), &installs),
            Err(RSelectError::PinnedNotFound("3.6".to_string()))
        );
    }

    fn ver(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    #[test]
    fn select_error_displays_bilingually() {
        assert!(RSelectError::NoRInstalled.to_string().contains("no R"));
        let e = RSelectError::PinnedNotFound("3.6".to_string());
        assert!(e.to_string().contains("3.6"));
    }
}
