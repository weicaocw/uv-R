# Changelog

## v0.4.0 — 多仓库 / Multi-repo

**中文**
- **多仓库（模块 H）**：合并多个仓库的 `PACKAGES` 后求解，解开跨仓库依赖（如 r-universe 包依赖另一仓库的包）；`uvr lock`/`install` 支持多个 `--repo`；安装时按每个包**自己的仓库**下载。
- **质量**：31 个单元测试；CI 全绿。新增教学课 24–25。

**English**
- **Multi-repo (Module H)**: resolve across merged repository indexes, fixing cross-repo dependencies; `uvr lock`/`install` accept multiple `--repo`; each package is downloaded from its own repo.
- **Quality**: 31 unit tests; CI green. New lessons 24–25.

## v0.3.0 — pubgrub 回溯求解器 / pubgrub backtracking resolver

**中文**
- **接入 pubgrub 0.4**（模块 D′）：用 `OfflineDependencyProvider` 把依赖图交给工业级求解器；约束转 `Ranges`，`Version` 实现与 `Eq` 一致的 `Hash`。
- **默认启用回溯求解**：`uvr lock` / `uvr install` 改用 `resolve_pubgrub`，能解开手写贪心版会误判为冲突的情形。手写贪心求解器保留作对照与教学。
- **质量**：28 个单元测试（含 pubgrub 一致性与回溯用例）；CI 全绿；端到端 demo 照常。
- 新增教学课 22–23。

**English**
- **Integrate pubgrub 0.4** (Module D'): hand the dependency graph to the industrial resolver via `OfflineDependencyProvider`; constraints → `Ranges`; `Version` implements a `Hash` consistent with its `Eq`.
- **Backtracking by default**: `uvr lock` / `uvr install` use `resolve_pubgrub`, solving cases the greedy resolver falsely reports as conflicts. The hand-written greedy resolver is kept for comparison/teaching.
- **Quality**: 28 unit tests (incl. pubgrub agreement & backtracking); CI green; end-to-end demos unaffected.
- New lessons 22–23.

## v0.2.0 — 联网、安装、benchmark / Networking, install, benchmark

**中文**
- **联网（模块 C）**：用 ureq 从真实 R 仓库抓 `PACKAGES`；`uvr lock --repo <url> <包>` 联网求解；跳过随 R 自带的 base/recommended 包。
- **下载 + 安装（模块 E）**：`uvr install --repo <url> [--lib <目录>] <包>...` 下载源码 tarball 并用 `R CMD INSTALL` 装进**项目本地库**（`-l` 隔离，不碰全局 R）。
- **Benchmark（模块 G）**：自写计时脚本 `scripts/bench.sh` + 诚实报告 `BENCHMARK.md`——一次性解析 uvr ~5 ms vs pak ~5.2 s（结构性优势：无 R 启动 / 无每进程重载元数据）；安装诚实报"打平"。
- **质量**：26 个单元测试（外加联网 / 安装的 `#[ignore]` 测试，手动跑）；CI 全绿。
- 新增教学课 17–21；课程七章全部完成。

**English**
- **Networking (Module C)**: fetch `PACKAGES` from real R repositories via ureq; `uvr lock --repo <url> <pkg>`; skip R's bundled base/recommended packages.
- **Download + install (Module E)**: `uvr install --repo <url> [--lib <dir>] <pkg>...` downloads source tarballs and `R CMD INSTALL`s them into a **project-local library** (isolated via `-l`, never the global R).
- **Benchmark (Module G)**: self-written `scripts/bench.sh` + an honest `BENCHMARK.md` — one-shot resolve uvr ~5 ms vs pak ~5.2 s (structural edge); installs are honestly a tie.
- **Quality**: 26 unit tests (+ `#[ignore]` network/install tests, run manually); CI green.
- New lessons 17–21; all seven chapters complete.

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
