# Changelog

## v0.13.0 — MD5 校验（兼容 CRAN）/ MD5 verification (CRAN)

**中文**
- **MD5 校验（模块 S）**：完整性校验现在也支持 **MD5**（CRAN 直连仓库在 `PACKAGES` 里给的是 `MD5sum`）。`verify_hash` 按算法前缀分派（`sha256:` / `md5:`），未知 / 无前缀则跳过。补齐了 v0.12 留下的"md5 暂跳过"口子。
- **质量**：69 个单元测试（+3 `#[ignore]`），fmt + clippy 全绿。演示：`lock praise`（CRAN，md5）→ `sync` 用 md5 校验后安装；篡改后 `sync` 检测到 md5 不符并拒装。新增依赖 `md-5`（项目本地）。新增教学课 45。

**English**
- **MD5 verification (Module S)**: integrity checks now also support **MD5** (CRAN's direct `PACKAGES` provides `MD5sum`). `verify_hash` dispatches on the algorithm prefix (`sha256:` / `md5:`), skipping unknown / prefix-less hashes. Closes the "md5 skipped for now" gap left by v0.12.
- **Quality**: 69 unit tests (+3 `#[ignore]`), fmt + clippy clean. Demo: `lock praise` (CRAN, md5) → `sync` verifies via md5 then installs; after tampering, `sync` detects the md5 mismatch and refuses. New dependency `md-5` (project-local). New lesson 45.

## v0.12.0 — 完整性校验 / Integrity verification

**中文**
- **校验和（模块 R）**：lockfile **v3** 记录每个包的校验和（仓库声明的 `SHA256`，回退 `MD5sum`）。`install` / `sync` 下载后、安装前**校验 SHA256**，不符即报错拒装——防传输损坏 / 篡改。空或非 sha256（CRAN 的 md5）暂跳过（记录但不阻断）。
- **可复现且可信**：锁版本（防漂移）+ 锁来源（v2 自包含）+ 锁校验和（防篡改）。新增依赖 `sha2`（项目本地，不污染系统）。
- **质量**：69 个单元测试（+3 `#[ignore]`），fmt + clippy 全绿。演示：篡改缓存 tarball 后 `sync` 检测到校验和不符并拒装。新增教学课 43–44。

**English**
- **Checksums (Module R)**: lockfile **v3** records each package's checksum (the repo-declared `SHA256`, falling back to `MD5sum`). `install` / `sync` **verify SHA256** after download and before install, erroring out on a mismatch — guarding against corruption/tampering. Empty or non-sha256 (CRAN's md5) is skipped for now (recorded, not enforced).
- **Reproducible and trustworthy**: lock the version (no drift) + lock the source (v2 self-contained) + lock the checksum (no tampering). New dependency `sha2` (project-local, no system pollution).
- **Quality**: 69 unit tests (+3 `#[ignore]`), fmt + clippy clean. Demo: after tampering with a cached tarball, `sync` detects the checksum mismatch and refuses to install. New lessons 43–44.

## v0.11.0 — 拓扑序安装 / Topological install order

**中文**
- **拓扑序安装（模块 Q）**：`install` / `sync` 现在按**依赖顺序**安装（被依赖的在前），而非字母序——保证 `R CMD INSTALL` 时依赖已就绪。用 Kahn 算法，就绪集合取名字最小者**确定可复现**；只看本批集合内的依赖边，自动忽略 base 包；成环兜底、绝不丢包；索引信息不足（自包含 sync）时优雅退化为名字序。
- **质量**：66 个单元测试（+3 `#[ignore]`），fmt + clippy 全绿。新增教学课 42（拓扑排序）。

**English**
- **Topological install order (Module Q)**: `install` / `sync` now install in **dependency order** (dependencies first), not alphabetically — so `R CMD INSTALL` always sees its deps ready. Kahn's algorithm, picking the lexicographically smallest ready node for **determinism**; only in-set edges (base packages auto-ignored); cycle-safe (never drops a package); gracefully degrades to name order when the index is unavailable (self-contained sync).
- **Quality**: 66 unit tests (+3 `#[ignore]`), fmt + clippy clean. New lesson 42 (topological sort).

## v0.10.0 — 自包含 lockfile v2 / Self-contained lockfile v2

**中文**
- **lockfile v2（模块 P）**：锁文件多记一列**来源仓库**（`name version repo`）。`uvr lock` 现产出 v2；解析**向后兼容** v1（两列）。
- **`uvr sync` 自包含**：`--repo` 变**可选**——v2 锁文件自带来源，一句 `uvr sync` 即可还原（对标 `cargo build` 读 `Cargo.lock`、`npm ci`）。旧 v1 锁文件 + `--repo` 仍可用。
- **质量**：62 个单元测试（+3 `#[ignore]`），fmt + clippy 全绿。本机演示 `lock → sync`（**不带 `--repo`**）还原 dotenv 1.0.3.9000。新增教学课 40–41。

**English**
- **lockfile v2 (Module P)**: the lockfile records a third column, the **source repo** (`name version repo`). `uvr lock` now emits v2; parsing is **backward-compatible** with v1 (two columns).
- **Self-contained `uvr sync`**: `--repo` is now **optional** — a v2 lockfile carries its sources, so a bare `uvr sync` restores (like `cargo build` reading `Cargo.lock`, `npm ci`). Old v1 lockfiles + `--repo` still work.
- **Quality**: 62 unit tests (+3 `#[ignore]`), fmt + clippy clean. Demo restores dotenv 1.0.3.9000 via `lock → sync` **without `--repo`**. New lessons 40–41.

## v0.9.0 — 并行下载 / Parallel downloads

**中文**
- **并行下载（模块 K）**：`install` / `sync` 现在**并行预取**所有 tarball，再串行 `R CMD INSTALL`。用作用域线程（`thread::scope`）+ 共享游标做工作窃取，自动负载均衡。`--jobs <N>` 控制并发度，默认 = CPU 核数。
- **可测的并行**：把"对每个包做什么"作为闭包注入 `parallel_for_each`，于是并行编排**无需网络**即可单测（全做 / 并发度 / 报错传播 / 空集）。
- **质量**：59 个单元测试（+3 `#[ignore]`），fmt + clippy 全绿。新增教学课 38–39。

**English**
- **Parallel downloads (Module K)**: `install` / `sync` now **prefetch** all tarballs in parallel, then `R CMD INSTALL` serially. Scoped threads (`thread::scope`) + a shared cursor for work-stealing (auto load-balancing). `--jobs <N>` sets the concurrency, default = CPU count.
- **Testable parallelism**: the per-package work is an injected closure in `parallel_for_each`, so the orchestration is unit-tested **without network** (all-run / concurrency / error propagation / empty).
- **Quality**: 59 unit tests (+3 `#[ignore]`), fmt + clippy clean. New lessons 38–39.

## v0.8.0 — lockfile 驱动的 sync / Lockfile-driven sync

**中文**
- **`uvr sync`（模块 O）**：按 lockfile **一键还原**环境——读 `uvr.lock`（或指定文件），**不求解**、严格安装锁定的版本（防漂移），下载并装进项目本地库。典型工作流：`uvr lock --repo ... pkg > uvr.lock` 提交进库 → 队友 / CI / 新机器 `uvr sync --repo ...` 还原一模一样的依赖（对标 `renv::restore` / `uv sync` / `npm ci`）。
- **重构**：抽出 `run_plan`（下载 + `R CMD INSTALL`）供 install/sync 共用；抽出 `resolve_r_bin` 让两者"用哪个 R"行为一致。
- **质量**：55 个单元测试（+3 `#[ignore]`），fmt + clippy 全绿。本机演示 lock→sync 还原 dotenv 1.0.3.9000。新增教学课 36–37。

**English**
- **`uvr sync` (Module O)**: restore an environment from a lockfile in one shot — read `uvr.lock` (or a given file), **without resolving**, install the exact locked versions (no drift), into a project-local lib. Workflow: `uvr lock --repo ... pkg > uvr.lock` (commit it) → teammates / CI / a fresh machine run `uvr sync --repo ...` to restore identical deps (analogue of `renv::restore` / `uv sync` / `npm ci`).
- **Refactor**: extract `run_plan` (download + `R CMD INSTALL`) shared by install/sync; extract `resolve_r_bin` so both pick the same R.
- **Quality**: 55 unit tests (+3 `#[ignore]`), fmt + clippy clean. Demo restores dotenv 1.0.3.9000 via lock→sync. New lessons 36–37.

## v0.7.0 — R 版本管理 + 用户手册 / R version management + user manual

**中文**
- **R 版本管理（模块 M）**：对标 uv 的 `uv python` 家族。`uvr r list`（发现本机所有 R）、`uvr r which`（按 pin > 最高 解析当前会用的 R）、`uvr r pin [版本]`（写 `.R-version`，前缀匹配如 `4.5`→`4.5.2`）、`uvr r install <版本>`（不自己装，委托 rig 或给 CRAN 指引）。`uvr install` 现在用解析出的那个 R 跑 `R CMD INSTALL`，pin 了没装则报错中止。
- **用户手册（模块 N）**：新增中英对照 `docs/MANUAL.md`（安装 / 命令参考 / R 版本管理 / 缓存模型 / 项目布局 / 排错 / 设计边界）；README 刷新到 v0.7。
- **质量**：52 个单元测试（+3 `#[ignore]` 真·R/网络），fmt + clippy 全绿；安装仅进项目本地库（`-l`）。新增教学课 29–35。

**English**
- **R version management (Module M)**: the analogue of uv's `uv python` family. `uvr r list` (discover all R's), `uvr r which` (resolve the R to use by pin > highest), `uvr r pin [version]` (write `.R-version`, prefix match e.g. `4.5`→`4.5.2`), `uvr r install <version>` (no self-install; delegates to rig or points to CRAN). `uvr install` now runs `R CMD INSTALL` with the resolved R and aborts if a pinned R is missing.
- **User manual (Module N)**: new bilingual `docs/MANUAL.md` (install / command reference / R version management / caching / layout / troubleshooting / design limits); README refreshed to v0.7.
- **Quality**: 52 unit tests (+3 `#[ignore]` real-R/network), fmt + clippy clean; installs only into a project-local lib (`-l`). New lessons 29–35.

## v0.6.0 — 端到端 benchmark vs pak / End-to-end benchmark vs pak

**中文**
- **端到端 benchmark（模块 L）**：uvr 与 pak 的多轴公平对照——解析 6ms vs 5415ms；暖缓存 lock 6ms；端到端装 dotenv uvr 暖 1575ms vs pak 5478ms（uvr 冷也胜 pak 暖 ~2.6×）。`R CMD INSTALL` 是共享地板（打平）；含编译大包待 binary 支持。
- `BENCHMARK.md` 重写为多轴诚实报告；`scripts/bench.sh` 加端到端安装段。新增教学课 28。

**English**
- **End-to-end benchmark (Module L)**: a fair, multi-axis comparison vs pak — resolve 6ms vs 5415ms; warm-cache lock 6ms; end-to-end install dotenv uvr warm 1575ms vs pak warm 5478ms (uvr cold beats pak warm ~2.6×). `R CMD INSTALL` is a shared floor; compiled packages await binary support.
- `BENCHMARK.md` rewritten as a multi-axis honest report; `scripts/bench.sh` gains an end-to-end install section. New lesson 28.

## v0.5.0 — 暖缓存 / Warm cache

**中文**
- **缓存（模块 I）**：抓到的 `PACKAGES` 与下载的 tarball 缓存到 `.uvr-cache/`；重复 `lock`/`install` 走暖缓存、免联网。实测 `lock --repo` 冷 640ms → 暖 6.8ms（~94×）。
- 新增教学课 26–27；`scripts/bench.sh` 加 cold/warm 对照。

**English**
- **Caching (Module I)**: cache `PACKAGES` and tarballs under `.uvr-cache/`; repeated `lock`/`install` hit the warm cache (no network). Measured `lock --repo` cold 640ms → warm 6.8ms (~94×).
- New lessons 26–27; `scripts/bench.sh` gains a cold/warm comparison.

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
