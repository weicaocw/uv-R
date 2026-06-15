# Step 19：下载 + `R CMD INSTALL` 到项目本地库

> 模块：E 下载+安装 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-19-download-install.md`）｜ 测试：✅ 25 passed + 真·安装 ｜ 上一步：[Step 18](step-18-builtin-skip-and-repo.md)

## 0. 一句话目标
把一个包的源码 tarball 下载下来，用 `R CMD INSTALL` 装进**项目本地库目录**（绝不碰全局 R 环境）。

## 1. 前置回顾
模块 C 已能联网抓元数据并求解。但"解出版本"还不等于"装上"。本步实现真正的安装：下载 + 委托 `R CMD INSTALL` 构建。

## 2. 先写测试（TDD·红）
纯逻辑（构造 tarball URL）可单测：
```rust
#[test]
fn builds_tarball_url() {
    assert_eq!(
        tarball_url("https://cran.r-project.org/", "praise", "1.0.0"),
        "https://cran.r-project.org/src/contrib/praise_1.0.0.tar.gz");
}
```
"下载 + 跑 R" 是 IO + 子进程、依赖网络与 R，用 **`#[ignore]`** 测试，手动跑。

## 3. 实现到通过（TDD·绿）
```rust
pub fn download(url: &str, dest: &Path) -> Result<(), String> {
    let resp = ureq::get(url).call().map_err(|e| e.to_string())?;
    let mut reader = resp.into_reader();                 // 流式，省内存
    let mut file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    std::io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn install_tarball(tarball: &Path, lib_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(lib_dir).map_err(|e| e.to_string())?;
    let status = Command::new("R")
        .arg("CMD").arg("INSTALL").arg("-l").arg(lib_dir).arg(tarball)
        .status().map_err(|e| e.to_string())?;
    if status.success() { Ok(()) } else { Err(format!("R CMD INSTALL 退出码 {:?}", status.code())) }
}
```
常规 `cargo test` → ✅ 25 passed。手动跑真·安装（下载 `praise` 装进 `target/test-r-lib`）：
```
$ cargo test -- --ignored installs_praise_locally
test ... ok  (2.14s)  →  target/test-r-lib/praise/DESCRIPTION 出现
```
并验证隔离：默认 `.libPaths()` 不含 `target/test-r-lib`，`-l` 把安装限定在我们的目录——**没碰全局库**。

## 4. 改了哪些文件 / 加了什么
- 新增 `src/install.rs`：`tarball_url`、`download`、`install_tarball` + 测试（含 1 个 `#[ignore]` 真·安装测试）。
- `src/lib.rs`：新增 `pub mod install;`。

## 5. 学到的语法 / 技巧
- **`std::process::Command`**：启动外部进程。`Command::new("R").arg("CMD").arg("INSTALL")...status()` 跑 `R CMD INSTALL` 并拿退出状态。
- **`resp.into_reader()` + `std::io::copy`**：把 HTTP 响应**流式**写进文件，不必整包读进内存。
- **`std::fs::File::create` / `create_dir_all`**：建文件 / 建目录（含父目录）。
- **`&Path` / `PathBuf`**：路径类型；`lib_dir.join("praise")` 拼路径。

## 6. 语言设计巧思
- **委托而非重造**：包的实际构建（可能编译 C/C++/Fortran）交给 R 自己的 `R CMD INSTALL`——正如 uv 不自己实现 Python 编译。我们只做"下载 + 编排"，把不可绕的构建留给 R。
- **`-l <lib_dir>` 是隔离的关键**：它把安装目标钉死在我们给的目录，因此**项目本地、不污染全局**——这与本机"绝不污染全局环境"的铁律一致，也正是 uv/renv 的项目级隔离思想。

## 7. 领域知识
- **`R CMD INSTALL`**：R 安装包的标准命令；吃一个源码 tarball，编译（如需）并安装到库目录。
- **项目本地库 vs 全局库**：R 用 `.libPaths()` 决定到哪找/装包。`-l` 指定一个不在默认 `.libPaths()` 里的目录，即"项目本地库"——多项目互不干扰、可复现。
- **源码 tarball**：`<name>_<version>.tar.gz`，放在仓库的 `src/contrib/` 下。

## 8. 软件设计理念
- **划清委托边界**：自己负责"决定装什么、从哪下、装到哪"，把"怎么构建一个 R 包"整体委托给 R。边界清晰，职责单一。
- **不污染环境**：默认装进项目本地目录，是对"可复现、可隔离"的工程承诺。

## 9. 小测验（自测）
1. 为什么用 `R CMD INSTALL` 而不自己解压/编译包？
2. `-l <dir>` 起什么作用？它如何保证不污染全局 R 库？
3. `into_reader()` + `io::copy` 相比"整包读进内存再写"有什么好处？
4. `Command::new("R")...status()` 返回什么？我们据此怎么判断成败？

## 10. 参考答案
1. 因为构建一个 R 包（可能含 C/C++/Fortran 编译、配置脚本、帮助文档生成）很复杂且必须遵循 R 的规则；委托给 `R CMD INSTALL` 既正确又省事，正如 uv 不自己实现 Python 编译。
2. 它把安装目标限定到指定目录；该目录不在默认 `.libPaths()` 里，故安装只落在项目本地、不进系统/用户全局库。
3. 流式拷贝边读边写，内存占用恒定、能处理大文件；整包读进内存则占用与文件等大、对大包不友好。
4. 返回进程的退出状态 `ExitStatus`；`status.success()` 为真表示安装成功，否则取 `status.code()` 报告失败退出码。

## 11. 下一步预告
Step 20：把它接进 CLI——`uvr install --repo <url> [--lib <dir>] <包>...`：解析依赖、逐个下载并安装进项目本地库。一个真实的小包 `dotenv` 将被端到端装上。
