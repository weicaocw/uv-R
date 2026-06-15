# Step 16：最小命令行（`uvr lock`）

> 模块：F 命令行 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-16-cli.md`）｜ 测试：✅ 22 passed + 端到端运行 ｜ 上一步：[Step 15](step-15-lockfile.md)

## 0. 一句话目标
把库里的"读 PACKAGES → 求解 → 渲染 lockfile"串成一个能用的命令：`uvr lock <PACKAGES 文件> <根包>...`。

## 1. 前置回顾
模块 A/B/D 已经把核心逻辑（版本、元数据、求解、lockfile）都做好且测试覆盖。本步加一层**薄薄的命令行外壳**，让人能从终端真正用上它——uvr 第一次成为可运行的工具。

## 2. 先写测试（TDD·红）
把命令的"业务核心"做成**纯函数**（文本进、文本出），便于测试：
```rust
#[test]
fn locks_package_and_deps() {
    let text = lock_from_packages(PACKAGES, &["pkgB".to_string()]).unwrap();
    assert!(text.contains("pkgB 2.0.0"));
    assert!(text.contains("pkgA 1.2.0"));
}
```
（`lock_from_packages` 未实现时编译失败；红从略。）

## 3. 实现到通过（TDD·绿）
核心（`src/commands.rs`，纯函数、可测）：
```rust
pub fn lock_from_packages(packages_text: &str, roots: &[String]) -> Result<String, ResolveError> {
    let index = PackageIndex::from_packages_file(packages_text);
    let mut combined: BTreeMap<String, Version> = BTreeMap::new();
    for root in roots {
        combined.extend(resolve(&index, root)?);
    }
    Ok(lockfile::render(&combined))
}
```
外壳（`src/main.rs`，只管参数与文件 IO）：
```rust
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("lock") if args.len() >= 4 => lock(&args[2], &args[3..]),
        _ => { eprintln!("用法 / usage: uvr lock <PACKAGES-file> <root-package>..."); ExitCode::FAILURE }
    }
}
```
`cargo test` → ✅ 22 passed。**端到端真实运行**：
```
$ cargo run -- lock testdata/PACKAGES pkgC
# uvr lockfile v1
pkgA 1.2.0
pkgB 2.0.0
pkgC 0.5.0
```
`pkgC` → `pkgB` → `pkgA`，自动解出整条依赖链并锁定版本。

## 4. 改了哪些文件 / 加了什么
- 新增 `src/commands.rs`：`lock_from_packages`（纯函数核心）+ 2 个测试。
- 重写 `src/main.rs`：解析 `args`、读文件、调用核心、按结果设进程退出码。
- `src/lib.rs`：新增 `pub mod commands;`。
- 新增 `testdata/PACKAGES`：演示用的样本仓库索引。

## 5. 学到的语法 / 技巧
- **`std::env::args()`**：取命令行参数（`args[0]` 是程序名）。
- **`std::fs::read_to_string(path)`**：把文件读成 `String`，返回 `Result`。
- **`fn main() -> ExitCode`**：`main` 可返回 [`std::process::ExitCode`]，用 `SUCCESS` / `FAILURE` 设进程退出码（脚本/CI 据此判断成败）。
- **`match args.get(1).map(String::as_str) { Some("lock") if ... => ... }`**：对可选参数做带卫语句的匹配。
- **`eprintln!` vs `print!`**：错误走标准错误（stderr），正常结果走标准输出（stdout）。
- **`BTreeMap::extend`**：把一个 map 的条目并入另一个。

## 6. 语言设计巧思
- **纯函数核心 + 薄 IO 外壳**：业务逻辑 `lock_from_packages` 不碰文件、不碰参数，输入输出都是数据 → 可单测、可复用；`main` 只做"读参数/读文件/写输出/设退出码"。这让绝大部分逻辑都在测试保护之下。
- **`ExitCode` 让失败可被脚本感知**：返回 `FAILURE` 时进程退出码非 0，CI 与 shell 能据此判断——比"打印一行错误就当没事"更严谨。

## 7. 领域知识
- **`lock` 子命令**：对应 `cargo generate-lockfile`、`uv lock` 等——从依赖声明解出确切版本并写 lockfile。我们的 `uvr lock` 从一个 `PACKAGES` 文件离线完成这件事。
- 真实世界里 `PACKAGES` 来自远端仓库（CRAN/PPM），需联网抓取（模块 C，资源墙）；这里用本地 `testdata/PACKAGES` 演示同样的核心流程。

## 8. 软件设计理念
- **端口与适配器（六边形架构）的雏形**：核心逻辑与外部世界（命令行、文件系统）隔离，外部只是"适配器"。核心不依赖 IO，于是易测、易移植（将来换成"从网络读 PACKAGES"只需换适配器）。
- **薄 main**：入口越薄越好，真正的活在库里。

## 9. 小测验（自测）
1. 为什么把 `lock_from_packages` 做成不碰文件的纯函数？
2. `fn main() -> ExitCode` 相比 `fn main()` 多了什么能力？
3. `eprintln!` 和 `print!` 分别往哪里输出？为什么要分开？
4. 真实世界里 `PACKAGES` 从哪来？为什么本步用本地文件？

## 10. 参考答案
1. 纯函数（文本进、文本出）便于单元测试与复用，把难测的文件/参数 IO 隔离在 `main`；绝大多数逻辑因此都能被测试覆盖。
2. 它能设置**进程退出码**（`SUCCESS`/`FAILURE`），让 shell 脚本与 CI 据此判断命令成功与否。
3. `eprintln!` → 标准错误（stderr），`print!` → 标准输出（stdout）。分开后，正常结果可被管道/重定向，错误信息不会污染数据输出。
4. 来自远端仓库（CRAN / Posit Package Manager），需联网抓取；联网属当前环境的资源墙，故本步用本地 `testdata/PACKAGES` 演示同一套核心流程。

## 11. 下一步预告
模块 F 完成：uvr 已是一个**能用的离线依赖求解器**（`uvr lock`）。接着开 PR 合入 `main`，并打上 **v0.1** 标签作为"可发布成品"。
此后的模块 C（联网抓 `PACKAGES`）、E（下载 / 安装 R 包）、G（对 pak 跑 benchmark）需要实时网络 / R / 系统安装——属本环境的**资源墙**，会在发布后清单交还给你。
