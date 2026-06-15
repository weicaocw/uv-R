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

## 构建与测试 / Build & test

```sh
cargo test    # 跑测试 / run the tests
cargo run     # 运行 / run the binary
```
