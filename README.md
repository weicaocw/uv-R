# uvr — 用 Rust 学着造一个「R 版 uv」

> A learn-by-building project: a uv/Cargo-style package manager for R, written in Rust.

本仓库是一个**手把手教学项目**。以零基础视角，通过亲手构建 `uvr`（一个用 Rust 写的、
类似 [uv](https://github.com/astral-sh/uv) / Cargo 的 R 语言包管理器），系统地学习
**Rust 语言、R 包管理、软件设计**。

- 每一小步都遵循 TDD：失败的测试 → 最小实现 → 通过。
- 每一小步都有一篇**自包含的简体中文教学课**：`docs/lessons/step-NN-*.md`（共 21 课）。
- 课程地图与进度见 `docs/CURRICULUM.md`。
- 提交与 PR 信息**中英双语**，便于事后翻历史复习。

This repository is a hands-on tutorial: learn Rust, R package management, and software design
by building `uvr`, a uv/Cargo-style package manager for R. Every tiny step is test-driven and
documented as a self-contained Chinese lesson under `docs/lessons/`.

## 现在能做什么（v0.2）/ What works now (v0.2)

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
installed dotenv 1.0.3.9000
→ 已安装到项目本地库 / installed into project-local lib: ./r-lib
```

**对 pak 的诚实 benchmark / honest benchmark vs pak**：见 [`BENCHMARK.md`](BENCHMARK.md)
（一次性解析：uvr ~5 ms vs pak ~5.2 s；安装这类重活诚实报"打平"）。

全部完成：版本模型 · 元数据(DCF/依赖图) · 联网 · 依赖求解（**pubgrub 回溯**，手写贪心版作对照） · 下载安装 · CLI · benchmark。
28 个单元测试 + CI（fmt / clippy / build / test）。

## 路线图 / Roadmap (v0.4+)

- ✅ 工业级 [`pubgrub`](https://github.com/pubgrub-rs/pubgrub) 回溯求解器（v0.3）。
- ✅ 合并多仓库索引、跨仓库依赖（v0.4）。
- 元数据 / 下载缓存（暖缓存）· binary 包优先（免编译）· 并行下载安装 —— 目标：能与 pak 做**端到端**公平对比。

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
bash scripts/bench.sh                               # 对 pak 跑 benchmark / benchmark vs pak
```
