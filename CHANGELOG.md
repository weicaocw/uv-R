# Changelog

## v0.1.0 — 离线依赖求解器 / Offline dependency resolver

**中文**
- **版本模型**：解析版本号；按数字逐段比较（零填充，`1.0 == 1.0.0`；`Eq`/`Ord` 一致）；版本约束 `>=`、`<=`、`==`、`<`、`>`。
- **元数据**：解析 R 仓库的 DCF（`PACKAGES` / `DESCRIPTION`）；解析依赖字段；建立包索引（依赖图）。
- **依赖求解（手写教学版）**：选满足约束的最高版本、递归传递依赖、冲突检测（`NotFound`/`Unsatisfiable`/`Conflict`）、lockfile（确定性、可往返）。
- **命令行**：`uvr lock <PACKAGES 文件> <根包>...` —— 离线读取仓库索引、求解、输出 lockfile。
- **质量**：22 个单元测试；CI 跑 fmt + clippy + build + test，全绿。
- 每一步都有自包含的简体中文教学课（`docs/lessons/`），提交 / PR 中英双语。

**English**
- **Version model**: parse versions; numeric component-wise comparison (zero-padded, `1.0 == 1.0.0`; `Eq`/`Ord` consistent); constraints `>=`, `<=`, `==`, `<`, `>`.
- **Metadata**: parse R's DCF (`PACKAGES` / `DESCRIPTION`); parse dependency fields; build a package index (dependency graph).
- **Dependency resolution (teaching-grade)**: pick highest satisfying version, recurse over transitive deps, detect conflicts (`NotFound`/`Unsatisfiable`/`Conflict`), write a deterministic round-trippable lockfile.
- **CLI**: `uvr lock <PACKAGES-file> <root-package>...` — resolve offline from a repository index and emit a lockfile.
- **Quality**: 22 unit tests; CI runs fmt + clippy + build + test, all green.
- Every step has a self-contained Simplified-Chinese lesson (`docs/lessons/`); commits / PRs are bilingual.

**未完成（资源墙）/ Not yet (resource walls)**
- 模块 C 联网抓取 `PACKAGES`（需实时网络 / 外部 crate）。
- 模块 E 下载与安装 R 包（需 R 与系统工具链）。
- 模块 G 对 `pak` 跑 benchmark（需 R、pak、`hyperfine`）。
- 模块 D 升级为 `pubgrub` 求解器（需 cargo 联网拉 crate）。
