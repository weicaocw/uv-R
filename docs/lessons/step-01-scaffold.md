# Step 01：项目骨架（cargo + hello world）

> 模块：A 版本模型 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-01-scaffold.md`）｜ 演示：✅ 通过 ｜ 上一步：（本模块第一步）

## 0. 一句话目标
用 Rust 官方工具 `cargo` 生成一个能编译运行的最小程序（打印 `Hello, world!`），把项目骨架立起来。

## 1. 前置回顾
本模块第一步，没有上一步。整体目标见 `docs/CURRICULUM.md`：用 Rust 造 `uvr`。万事开头，先要一个能跑的骨架。

## 2. 先写测试（TDD·红）
本步只是项目骨架，还没有任何"行为"可断言，写单元测试为时尚早。按惯例这一步用"**运行演示**"代替测试（从下一步起就有真正的测试了）。**本步不涉及 TDD 红阶段。**

## 3. 实现到通过（这里是"运行演示"）
骨架由 `cargo init` 生成，核心是 `src/main.rs`：

```rust
fn main() {
    println!("Hello, world!");
}
```

运行：

```
$ cargo run
   Compiling uvr v0.1.0
    Finished `dev` profile [unoptimized + debuginfo]
     Running `target/debug/uvr`
Hello, world!
```

✅ 能编译、能运行、能打印——骨架立住了。

## 4. 改了哪些文件 / 加了什么
- `Cargo.toml`：项目清单（包名、版本、edition、依赖列表）。
- `src/main.rs`：程序入口，打印一行。
- `Cargo.lock`：依赖锁定文件（现在没有外部依赖，内容很短；二进制项目按惯例提交它）。
- `.gitignore`：已在 `main` 基线中（忽略构建产物 `/target`）。

## 5. 学到的语法 / 技巧
- **`cargo`**：Rust 官方"构建工具 + 包管理器"。`cargo run` = 编译 + 运行；`cargo build` 只编译；`cargo test` 跑测试。
- **`Cargo.toml`**：TOML 格式的清单。`[package]` 段含 `name` / `version` / `edition`；`[dependencies]` 段放外部库（现在空）。
- **`fn main() { ... }`**：`fn` 定义函数；`main` 是程序入口（启动时第一个被调用）；`()` 表示无参数；`{}` 是函数体。
- **`println!(...)`**：打印并换行。末尾的 `!` 表示它是**宏**（编译期展开成代码，因而能接受可变数量参数、并在编译期检查格式串），不是普通函数。
- **`;`**：语句结束符。

## 6. 语言设计巧思
- **edition（版次）**：`Cargo.toml` 里的 `edition = "2024"` 让 Rust 在**不破坏旧代码**的前提下演进语法——每个包各选各的、还能互相依赖。这是 Rust "既能进化又不抛弃旧代码"的设计（对比 Python 2→3 的断裂之痛）。
- **debug vs release**：`cargo run` 默认 **debug** 档（不优化、编译快、跑得慢、带调试信息，产物在 `target/debug/`）。测速要用 `cargo build --release`（优化，产物在 `target/release/`）。**记住：任何性能数字都必须来自 release 构建**——这条在以后做 benchmark 时是底线。

## 7. 领域知识
本步不涉及 R 包管理的具体领域知识（先把工具链跑起来）。顺带一提：`Cargo.toml` 的 `[dependencies]` 之于 Rust，正对应 R 包 `DESCRIPTION` 文件里的 `Imports` 字段——"清单 + 锁文件 + 仓库 + 求解器"这套结构，正是我们要为 R 重造的东西。

## 8. 软件设计理念
- **可运行的骨架优先（walking skeleton）**：先让一个最小但**端到端能跑**的程序立起来，再往里长功能。好处是任何时刻都有一个绿色、可运行的基线，风险最低。
- **约定优于配置**：cargo 用固定目录约定（`src/main.rs`、`target/`），省去大量手工配置。

## 9. 小测验（自测）
1. `cargo run` 和 `cargo build` 有什么区别？
2. `println!` 后面的 `!` 意味着什么？它和普通函数有何不同？
3. 为什么测速时不能用默认的 `cargo run`，要加 `--release`？
4. `.gitignore` 里为什么要忽略 `target/`？

## 10. 参考答案
1. `cargo build` 只编译、生成可执行文件；`cargo run` 先编译再运行它。
2. `!` 表示这是一个**宏**：在编译前被展开成真正的代码。与函数不同，它能接受**可变数量**的参数，并能在**编译期**检查格式串（如占位符 `{}` 的个数对不对）。
3. 默认是 debug 档，不做优化、运行慢；用它测速会假性偏慢、结论失真。release 档开了优化，才反映真实性能。
4. `target/` 是编译产物：又大、又随机器变化、又能随时重新生成，提交进 git 既无意义又臃肿。

## 11. 下一步预告
Step 02：为"版本号"造第一个自定义类型 `Version`（一个 `struct`，里面用 `Vec` 装一串整数），并打印看看。开始接触 Rust 的类型系统。
