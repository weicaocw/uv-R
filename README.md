# uvr — 用 Rust 学着造一个「R 版 uv」

> A learn-by-building project: a uv/Cargo-style package manager for R, written in Rust.

本仓库是一个**手把手教学项目**。以零基础视角，通过亲手构建 `uvr`（一个用 Rust 写的、
类似 [uv](https://github.com/astral-sh/uv) / Cargo 的 R 语言包管理器），系统地学习
**Rust 语言、R 包管理、软件设计**。

- 每一小步都遵循 TDD：失败的测试 → 最小实现 → 通过。
- 每一小步都有一篇**自包含的简体中文教学课**：`docs/lessons/step-NN-*.md`。
- 课程地图与进度见 `docs/CURRICULUM.md`。
- 提交与 PR 信息**中英双语**，便于事后翻历史复习。

This repository is a hands-on tutorial: learn Rust, R package management, and software design
by building `uvr`, a uv/Cargo-style package manager for R. Every tiny step is test-driven and
documented as a self-contained Chinese lesson under `docs/lessons/`. See `docs/CURRICULUM.md`
for the roadmap.

## 现在能做什么（v0.1）/ What works now (v0.1)

v0.1 是一个**离线依赖求解器**：从一个 R 仓库的 `PACKAGES` 文件，解出某些包的全部传递依赖，并写出 lockfile。

```sh
$ cargo run -- lock testdata/PACKAGES pkgC
# uvr lockfile v1
pkgA 1.2.0
pkgB 2.0.0
pkgC 0.5.0
```

已实现：版本模型（解析 / 比较 / 约束）· 元数据（DCF 解析 / 依赖图）· 依赖求解（传递依赖 / 冲突检测 / lockfile）· 最小 CLI。22 个单元测试 + CI（fmt / clippy / build / test）。

v0.1 is an **offline dependency resolver**: from a repository `PACKAGES` file it resolves a
package's full transitive dependencies and writes a lockfile (see the command above).

## 路线图与资源墙 / Roadmap & resource walls

后续模块需要本机当前不具备的资源（详见 `docs/CURRICULUM.md`）：

- **C 联网**：抓取 CRAN / Posit Package Manager 的 `PACKAGES`（需实时网络 / 外部 crate）。
- **E 安装**：下载并安装 R 包（需 R 与系统工具链 / `R CMD INSTALL`）。
- **G Benchmark**：对 `pak` 跑对照实验（需 R、`pak`、`hyperfine`）。
- **D 升级**：把手写教学求解器换成工业级 `pubgrub`（需 cargo 联网拉 crate）。

## 怎么学 / How it's taught

每一步都有一篇自包含的简体中文教学课：`docs/lessons/step-NN-*.md`（学到的语法、语言设计巧思、
R 包管理知识、软件设计理念、改了哪些文件、过了哪些测试、对应提交、外加小测验 + 参考答案）。
课程地图见 `docs/CURRICULUM.md`。

## 构建与测试 / Build & test

```sh
cargo test                                  # 跑全部测试 / run all tests
cargo run -- lock <PACKAGES-file> <pkg>...  # 求解并输出 lockfile / resolve & print a lockfile
```
