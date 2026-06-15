# uvr — 用 Rust 学着造一个「R 版 uv」

> A learn-by-building project: a uv/Cargo-style package manager for R, written in Rust.

本仓库是一个**手把手教学项目**。以零基础视角，通过亲手构建 `uvr`（一个用 Rust 写的、
类似 [uv](https://github.com/astral-sh/uv) / Cargo 的 R 语言包管理器），系统地学习
**Rust 语言、R 包管理、软件设计**。

- 每一小步都遵循 TDD：失败的测试 → 最小实现 → 通过。
- 每一小步都有一篇**自包含的简体中文教学课**：`docs/lessons/step-NN-*.md`（共 42 课）。
- 课程地图与进度见 `docs/CURRICULUM.md`。
- 提交与 PR 信息**中英双语**，便于事后翻历史复习。

This repository is a hands-on tutorial: learn Rust, R package management, and software design
by building `uvr`, a uv/Cargo-style package manager for R. Every tiny step is test-driven and
documented as a self-contained Chinese lesson under `docs/lessons/`.

> 📖 **用户手册（中英对照）/ User manual (bilingual)**：[`docs/MANUAL.md`](docs/MANUAL.md)
> —— 面向使用的完整参考：命令、R 版本管理、缓存、项目布局、排错。

## 现在能做什么（v0.11）/ What works now (v0.11)

**离线求解 + 锁定 / offline resolve & lock**
```sh
$ cargo run -- lock testdata/PACKAGES pkgC
# uvr lockfile v1
pkgA 1.2.0
pkgB 2.0.0
pkgC 0.5.0
```

**联网求解（从真实仓库）/ resolve live from a real repo**
```sh
$ cargo run -- lock --repo https://jeroen.r-universe.dev jsonlite
# uvr lockfile v1
jsonlite 2.0.1
```

**安装到项目本地库（不碰全局 R）/ install into a project-local lib**
```sh
$ cargo run -- install --repo https://gaborcsardi.r-universe.dev dotenv --lib ./r-lib
→ 使用 R / using R 4.5.2: /usr/local/bin/R
installed dotenv 1.0.3.9000
→ 已安装到项目本地库 / installed into project-local lib: ./r-lib
```

**按 lockfile 还原环境（对标 `uv sync` / `renv::restore`）/ restore from a lockfile**
```sh
$ cargo run -- lock --repo https://gaborcsardi.r-universe.dev dotenv > uvr.lock
$ cat uvr.lock
# uvr lockfile v2
dotenv 1.0.3.9000 https://gaborcsardi.r-universe.dev   # v2 自带来源仓库 / records the source repo
$ cargo run -- sync --lib ./r-lib          # ← 无需 --repo！/ no --repo needed!
synced dotenv 1.0.3.9000   # 严格按锁定版本，不求解、不漂移 / exact locked version, no drift
```

**管理 R 版本（对标 `uv python`）/ manage R versions (like `uv python`)**
```sh
$ cargo run -- r list           # 发现本机所有 R（* = 当前选中）/ discover all R's (* = selected)
* 4.5.2 /usr/local/bin/R
$ cargo run -- r pin 4.5        # 钉到 ./.R-version（前缀匹配 4.5→4.5.2）/ pin to ./.R-version
$ cargo run -- r which          # 看当前项目会用哪个 R / which R this project uses
4.5.2 /usr/local/bin/R
```

**对 pak 的诚实 benchmark / honest benchmark vs pak**：见 [`BENCHMARK.md`](BENCHMARK.md)
（一次性解析：uvr ~5 ms vs pak ~5.2 s；安装这类重活诚实报"打平"）。

全部完成：版本模型 · 元数据(DCF/依赖图) · 联网 · 依赖求解（**pubgrub 回溯**，手写贪心版作对照） · 下载安装 · CLI · benchmark · 多仓库 · 暖缓存 · **R 版本管理** · **lockfile sync（v2 自包含）** · **并行下载** · **拓扑序安装**。
66 个单元测试 + CI（fmt / clippy / build / test）。

## 路线图 / Roadmap

- ✅ 工业级 [`pubgrub`](https://github.com/pubgrub-rs/pubgrub) 回溯求解器（v0.3）。
- ✅ 合并多仓库索引、跨仓库依赖（v0.4）。
- ✅ 元数据 / 下载缓存（暖缓存，v0.5）；端到端 benchmark vs pak（v0.6）。
- ✅ **R 版本管理**：发现 / `.R-version` 钉版本 / 选择 / 用选中的 R 装包（v0.7，对标 `uv python`）。
- ✅ **lockfile `sync`**：按 `uvr.lock` 一键还原、不求解防漂移（v0.8，对标 `uv sync` / `renv::restore`）。
- ✅ **并行下载**：`install` / `sync` 并行预取 tarball，`--jobs <N>` 控制并发（v0.9，作用域线程 + 工作窃取）。
- ✅ **lockfile v2 自包含**：锁文件记来源仓库，`uvr sync` 无需 `--repo`（v0.10，对标 `cargo build` / `npm ci`）。
- ✅ **拓扑序安装**：依赖先于依赖者（v0.11，Kahn 算法、确定性）。
- ⏭ binary 包优先（免编译，本环境受限）· 并行安装（按拓扑层）· 锁文件校验和（sha256）。

## 怎么学 / How it's taught

每一步都有一篇自包含的简体中文教学课：`docs/lessons/step-NN-*.md`（学到的语法、语言设计巧思、
R 包管理知识、软件设计理念、改了哪些文件、过了哪些测试、对应提交、外加小测验 + 参考答案）。
课程地图见 `docs/CURRICULUM.md`。

## 构建与测试 / Build & test

```sh
cargo test                                          # 跑全部测试 / run all tests
cargo run -- lock    <PACKAGES-file> <pkg>...        # 离线求解 / resolve offline
cargo run -- lock    --repo <repo-url> <pkg>...      # 联网求解 / resolve live
cargo run -- install --repo <repo-url> [--lib <dir>] <pkg>...  # 安装到本地库 / install locally
cargo run -- sync    --repo <repo-url> [--lib <dir>] [<lockfile>]  # 按 lockfile 还原 / restore
cargo run -- r list | which | pin [<ver>]           # 管理 R 版本 / manage R versions
bash scripts/bench.sh                               # 对 pak 跑 benchmark / benchmark vs pak
```

完整命令与用法见用户手册 [`docs/MANUAL.md`](docs/MANUAL.md)。/ Full command reference: [`docs/MANUAL.md`](docs/MANUAL.md).
