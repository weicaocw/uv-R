# Step 08：拆成库 crate + 模块系统

> 模块：B 元数据 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-08-lib-and-modules.md`）｜ 测试：✅ 6 passed（重构前后均绿）｜ 上一步：[Step 07](step-07-ci.md)

## 0. 一句话目标
把"版本"相关代码从 `main.rs` 搬进一个**库 crate**（`src/lib.rs` + `src/version.rs`），让 `main.rs` 变成调用库的薄壳。

## 1. 前置回顾
模块 A 把所有代码堆在 `main.rs` 里。马上要进入元数据解析，代码会变多；而且核心逻辑应当**可被复用、可被独立测试**。是时候引入 Rust 的**模块系统**，把项目分层。

## 2. 重构步的"测试"（安全网）
这一步**不新增功能、不新增测试**——它是"重构"。重构的安全网，是**已有的 6 个测试在重构后仍然全绿**。重构前 `cargo test` 是 6 passed；下面重构后再跑，仍应 6 passed，证明我们没改坏行为。

## 3. 实现（拆分）
- `src/version.rs`：放 `Version` / `Op` / `Constraint` 及其测试；对外要用的类型和方法加 `pub`。
- `src/lib.rs`：库 crate 根，声明模块：
  ```rust
  pub mod version;
  ```
- `src/main.rs`：薄壳，调用库：
  ```rust
  use uvr::version::{Constraint, Version};

  fn main() {
      let c = Constraint::parse(">= 1.2.0").unwrap();
      let v = Version::parse("1.10.0").unwrap();
      println!("{:?} 满足 >= 1.2.0 ? {}", v, c.matches(&v));
  }
  ```
重构后 `cargo test`：
```
Running unittests src/lib.rs ...
running 6 tests
test version::tests::parses_dotted_version ... ok
... (共 6 个) ...
test result: ok. 6 passed; 0 failed
```
✅ 行为不变、测试全绿——重构成功。

## 4. 改了哪些文件 / 加了什么
- 新增 `src/lib.rs`：库 crate 根，`pub mod version;`。
- 新增 `src/version.rs`：从 `main.rs` 迁来的版本模型，类型 / 方法标注 `pub`。
- `src/main.rs`：删去版本代码，改为 `use uvr::version::...` 的薄壳。

## 5. 学到的语法 / 技巧
- **库 crate vs 二进制 crate**：一个包可同时有 `src/lib.rs`（库，供复用 / 测试）和 `src/main.rs`（可执行）。二进制通过包名 `uvr` 使用库：`use uvr::version::...`。
- **`mod`**：声明模块。`pub mod version;` 表示"有一个公开模块 version，代码在 `version.rs`"。
- **`pub`（可见性）**：Rust 里**一切默认私有**；只有标了 `pub` 的项才能被模块外访问。所以 `Version`、`Constraint`、`parse`、`matches` 加了 `pub`，而 `Op`、字段 `parts` 保持私有（外部用不到）。
- **`use` 与路径**：`use uvr::version::{Constraint, Version};` 把长路径引入当前作用域。
- **`//!` 文档注释**：写在文件/模块顶部，描述这个模块是干什么的。

## 6. 语言设计巧思
- **默认私有 + 显式 `pub`**：Rust 用可见性强制**信息隐藏**——你能放心改私有实现（如 `parts` 的内部表示），不怕外部依赖它。模块的"对外接口"由 `pub` 精确划定。
- **lib/bin 分离**：把逻辑放进库，使其能被单元测试、被未来的其它二进制 / 集成测试复用；`main` 只做"组装"。这也是 uv 等成熟工具的结构。

## 7. 领域知识
本步是工程结构调整，不涉及 R 包管理领域知识。但它为下一步"解析 PACKAGES 元数据"腾出了清晰的模块位置（接下来会新增 `metadata` 模块）。

## 8. 软件设计理念
- **关注点分离 / 分层**：核心库（领域逻辑）与命令行壳（用户接口）分开，各自演化。
- **信息隐藏**：私有字段 / 私有枚举把实现细节关在模块内，缩小"出错面"。
- **为扩展预留结构**：在加新功能（元数据）前先把骨架理顺——这本身是一种"逐步求精"。

## 9. 小测验（自测）
1. 一个 Rust 包能同时有库和可执行文件吗？二进制怎么用库里的东西？
2. Rust 里一个类型/函数默认是公开还是私有？怎么让它公开？
3. 为什么 `Op` 和字段 `parts` 不加 `pub`？
4. 重构步没有新功能，它的"测试"指什么？

## 10. 参考答案
1. 能：`src/lib.rs` 是库、`src/main.rs` 是二进制；二进制用 `use 包名::模块::项`（这里 `use uvr::version::...`）来使用库。
2. 默认**私有**；加 `pub` 才公开。
3. 它们是实现细节，外部不需要直接访问；保持私有可以让我们将来自由改内部表示而不影响使用方（信息隐藏）。
4. 指"**已有测试在重构后仍然全绿**"——它保证我们改了结构却没改坏行为。

## 11. 下一步预告
Step 09：新增 `metadata` 模块，解析 R 仓库的 **DCF 格式**（`PACKAGES` / `DESCRIPTION` 文件的布局）。开始接触真正的 R 包管理数据，并学习 `HashMap`、字符串处理与"折行续行"的解析。
