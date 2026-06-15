# Step 17：联网抓取 PACKAGES（第一个外部依赖 ureq）

> 模块：C 联网 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-17-network-fetch.md`）｜ 测试：✅ 23 passed + 1 ignored（联网）｜ 上一步：[Step 16](step-16-cli.md)

## 0. 一句话目标
加一个网络层：由仓库基址构造 `PACKAGES` 的 URL，并真的 HTTP GET 把它抓下来。

## 1. 前置回顾
v0.1 是离线的（从本地 `testdata/PACKAGES` 求解）。要面向真实世界，得能从 CRAN / Posit Package Manager / r-universe **联网抓** `PACKAGES`。本步引入项目的**第一个外部 crate**（`ureq`）来做 HTTP。

## 2. 先写测试（TDD·红）
纯逻辑（构造 URL）可直接单测：
```rust
#[test]
fn builds_packages_url() {
    assert_eq!(packages_url("https://x.dev/"), "https://x.dev/src/contrib/PACKAGES");
}
```
而真正的"联网 GET"是 IO、且依赖网络，不适合放进常规测试（会让 CI 抖动）——我们给它一个 **`#[ignore]`** 的联网测试，手动跑。

## 3. 实现到通过（TDD·绿）
先加依赖：`cargo add ureq@2`（写进 `Cargo.toml` 的 `[dependencies]`，cargo 自动联网拉取并编译）。再写：
```rust
pub fn packages_url(repo_base: &str) -> String {
    format!("{}/src/contrib/PACKAGES", repo_base.trim_end_matches('/'))
}

pub fn get_text(url: &str) -> Result<String, FetchError> {
    ureq::get(url)
        .call().map_err(|e| FetchError(e.to_string()))?
        .into_string().map_err(|e| FetchError(e.to_string()))
}
```
常规 `cargo test` → ✅ 23 passed + 1 ignored。手动跑联网测试：
```
$ cargo test -- --ignored fetches_real_packages
test ... ok   （0.17s 抓到 https://jeroen.r-universe.dev 的真实 PACKAGES）
```

## 4. 改了哪些文件 / 加了什么
- `Cargo.toml`：`[dependencies]` 新增 `ureq = "2"`（首个外部依赖）。
- `Cargo.lock`：记录 ureq 及其传递依赖的精确版本。
- 新增 `src/fetch.rs`：`packages_url`、`get_text`、`FetchError` + 测试（含 1 个 `#[ignore]` 联网测试）。
- `src/lib.rs`：新增 `pub mod fetch;`。

## 5. 学到的语法 / 技巧
- **引入外部 crate**：`cargo add ureq@2` 写进 `[dependencies]`；`use` 或全路径 `ureq::get(...)` 调用。
- **`ureq::get(url).call()?.into_string()?`**：发请求、拿响应、读成字符串。
- **`map_err(|e| FetchError(e.to_string()))`**：把底层错误转换成**我们自己的**错误类型。
- **newtype 错误 `struct FetchError(pub String)` + `impl Display`**：一个轻量错误类型。
- **`trim_end_matches('/')`**：去掉尾部多余斜杠，URL 拼接更稳。
- **`#[ignore = "需要网络"]`**：标记默认跳过的测试，手动 `--ignored` 才跑。

## 6. 语言设计巧思
- **不让外部库类型泄漏进自己的 API**：`get_text` 返回 `Result<String, FetchError>` 而非 `ureq::Error`——这样将来换 HTTP 库（reqwest/...）时，调用方代码不受影响。**用自己的类型在边界处"翻译"外部错误**，是控制依赖耦合的关键技巧。
- **纯函数与 IO 分离**：`packages_url`（纯、可测）和 `get_text`（IO、`#[ignore]` 测）分开，绝大部分逻辑仍在确定性测试保护下。

## 7. 领域知识
- **R 仓库布局**：源码包索引在 `<repo>/src/contrib/PACKAGES`。CRAN、Posit Package Manager、r-universe、Bioconductor 都遵循这一布局。
- **`PACKAGES` ≈ PyPI 的 simple index**：一次抓取拿到该仓库全部包的元数据，喂给求解器。
- 真实 `PACKAGES` 里还有 `SHA256`、`File`（tarball 名）、`NeedsCompilation`、`SystemRequirements` 等字段——下载与安装（模块 E）会用到。

## 8. 软件设计理念
- **依赖管理首次"自食其力"**：我们往 `Cargo.toml` 的 `[dependencies]` 加了 `ureq`——这正是 R 包 `DESCRIPTION` 里 `Imports` 的对应物。我们在用"包管理"造"包管理器"。
- **适配器层 / 防腐层**：网络是外部世界，`fetch` 模块是隔离它的适配器；错误在边界处被翻译成内部类型，核心逻辑不被外部细节污染。

## 9. 小测验（自测）
1. 为什么"联网 GET"的测试要标 `#[ignore]`，而"构造 URL"的测试不用？
2. `get_text` 为什么返回自定义的 `FetchError`，而不是直接把 `ureq::Error` 抛出去？
3. `cargo add ureq@2` 改动了哪两个文件？各自记录什么？
4. R 仓库里 `PACKAGES` 文件在什么路径？它的作用类比 Python 的什么？

## 10. 参考答案
1. 联网测试依赖网络、不确定、会让 CI 抖动，故默认忽略、手动跑；构造 URL 是纯逻辑、确定，适合常规单测。
2. 为了不让 `ureq` 的类型渗入我们的公共 API；在边界处翻译成 `FetchError`，将来更换 HTTP 库时调用方无需改动（降低耦合）。
3. `Cargo.toml`（在 `[dependencies]` 声明 `ureq = "2"`）与 `Cargo.lock`（锁定 ureq 及其传递依赖的精确版本）。
4. 在 `<repo>/src/contrib/PACKAGES`；它类比 PyPI 的 simple index——仓库全部包的元数据清单，是依赖求解的原料。

## 11. 下一步预告
Step 18：让求解器**跳过随 R 自带的 base/recommended 包**（如 `utils`、`methods`），这样面向真实 `PACKAGES` 才解得动；并把 CLI 接上：`uvr lock --repo <url> <包>` 从真实仓库联网求解。
