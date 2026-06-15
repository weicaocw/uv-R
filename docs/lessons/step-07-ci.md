# Step 07：接上 CI（GitHub Actions）

> 模块：A 版本模型 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-07-ci.md`）｜ 测试：本地等价检查 ✅；CI 在 GitHub 上运行 ｜ 上一步：[Step 06](step-06-version-constraints.md)

## 0. 一句话目标
给项目接上 **CI（持续集成）**：每次 push 或开 PR，GitHub 自动在一台干净机器上跑"格式检查 + lint + 构建 + 测试"。

## 1. 前置回顾
模块 A 已经有了一套测试（6 个）。**正因为有测试可跑，现在加 CI 才有意义**——CI 一上来就能守住"主干始终是绿的"。

## 2. 先验证（本步的"测试"特殊）
CI 的"绿"是在 GitHub 上跑出来的，本地看不到。但我们可以**在本地先跑一遍等价检查**，确保推上去不会立刻变红：
```
$ cargo fmt --all -- --check    # exit 0
$ cargo clippy --all-targets -- -D warnings   # exit 0
$ cargo test                    # 6 passed
```
三项都过 → 可以放心让 CI 接管。

## 3. 实现（CI 配置）
新增 `.github/workflows/ci.yml`：
```yaml
name: CI
on:
  push:
  pull_request:
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo build --verbose
      - run: cargo test --verbose
```

## 4. 改了哪些文件 / 加了什么
- `.github/workflows/ci.yml`：新增 CI 工作流，跑 fmt / clippy / build / test 四道关。

## 5. 学到的语法 / 技巧（YAML 与 Actions）
- **`name`**：工作流名字。
- **`on:`**：触发条件。`push` 与 `pull_request` 表示"每次推送、每个 PR"都跑。
- **`jobs:`**：一组任务；这里只有一个 `test`。
- **`runs-on: ubuntu-latest`**：在一台全新的 Ubuntu 机器上跑（干净环境 = 可复现）。
- **`steps:`**：按顺序执行的步骤。
  - **`uses: 某/动作@版本`**：复用别人写好的"动作"。`actions/checkout` 把代码拉下来；`dtolnay/rust-toolchain` 装 Rust。
  - **`with:`**：给动作传参，这里要 `clippy, rustfmt` 两个组件。
  - **`run: 命令`**：直接跑一条 shell 命令。
- 任一步骤失败（非 0 退出码），整个 CI 标红。

## 6. 语言设计巧思 / 工程实践
- **`-D warnings`**：把"警告"提升为"错误"。平时警告容易被忽略；在 CI 里一刀切地拒绝任何警告，能持续保持代码整洁、防止"破窗"。
- **干净环境可复现**：CI 每次从零装环境、拉代码，避免"在我机器上是好的"这类问题——这正是我们项目要为 R 解决的可复现性，在此先用在自己身上。

## 7. 领域知识
本步聚焦工程化，不涉及 R 包管理领域知识。（理念相通：CI 的"干净环境 + 锁定工具链"与包管理追求的"可复现安装"是同一种思想。）

## 8. 软件设计理念
- **质量门（quality gate）与防回归**：把"该做的检查"自动化、强制化，挡在合并之前。主干因此始终可构建、可测试。
- **渐进式**：CI 也可以小步长大——目前是 fmt/clippy/build/test，以后可加缓存、多平台矩阵、发布产物等，每一项都能再成一课。

## 9. 小测验（自测）
1. `on: [push, pull_request]` 表示 CI 在什么时候运行？
2. `uses:` 和 `run:` 有什么区别？
3. `-D warnings` 起什么作用？为什么在 CI 里这么做有价值？
4. 为什么"先有测试，再加 CI"，而不是反过来？

## 10. 参考答案
1. 每次有人 push 提交、以及每次开 / 更新 PR 时，CI 都会运行。
2. `uses:` 复用一个现成的"动作"（如拉代码、装工具链）；`run:` 直接执行一条 shell 命令。
3. 它把所有编译 / clippy 警告当成错误，使任何警告都会让 CI 失败，从而强制保持零警告、防止代码质量缓慢退化。
4. 因为 CI 的核心价值是"自动跑测试守住绿色"；没有测试时 CI 几乎没东西可守，价值很小。先有测试，CI 才名副其实。

## 11. 下一步预告
模块 A（版本模型）到此完成：解析、比较、约束、CI 齐备。接下来开一条**中英双语 PR** 合入 `main`，并**暂停等待你 review**。之后进入 **模块 B：元数据**——解析 R 仓库的 `PACKAGES` 文件与依赖字段，开始接触真正的 R 包管理数据。
