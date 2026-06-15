# Step 18：跳过 R 自带包 + 联网求解（`uvr lock --repo`）

> 模块：C 联网 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-18-builtin-skip-and-repo.md`）｜ 测试：✅ 24 passed + 真·联网 demo ｜ 上一步：[Step 17](step-17-network-fetch.md)

## 0. 一句话目标
让求解器**跳过随 R 自带的 base/recommended 包**（如 `utils`、`methods`），并把 CLI 接上：`uvr lock --repo <仓库> <包>` 联网求解。

## 1. 前置回顾
Step 17 能抓真实 `PACKAGES` 了。但真实包大量依赖 `utils`、`methods`、`stats` 这些**随 R 发行、不在 `PACKAGES` 里**的包——直接求解会 `NotFound`。本步把这些自带包识别出来跳过，求解真实数据才走得通。

## 2. 先写测试（TDD·红）
```rust
#[test]
fn skips_builtin_packages() {
    let idx = PackageIndex::from_packages_file(
        "Package: pkgB\nVersion: 2.0.0\nDepends: R (>= 3.0.0), utils, methods\n");
    let res = resolve(&idx, "pkgB").unwrap();   // 不再因 utils/methods 而 NotFound
    assert!(!res.contains_key("utils"));
    assert!(!res.contains_key("methods"));
}
```
（加 `is_builtin` 之前，`utils` 会被当成缺失包 → `Err(NotFound("utils"))`，测试失败。）

## 3. 实现到通过（TDD·绿）
把"随 R 自带"的包名收成一个常量集合，求解时跳过：
```rust
const BUILTIN: &[&str] = &[
    "R", "base", "compiler", "datasets", "graphics", "grDevices", "grid", "methods",
    "parallel", "splines", "stats", "stats4", "tcltk", "tools", "utils", "translations",
    // recommended：boot class cluster codetools foreign KernSmooth lattice MASS Matrix
    //              mgcv nlme nnet rpart spatial survival
];
pub fn is_builtin(name: &str) -> bool { BUILTIN.contains(&name) }
```
求解循环里把 `if name == "R"` 换成 `if is_builtin(&name)`。再给 CLI 加 `--repo` 分支：
```rust
Some("--repo") if rest.len() >= 3 => {
    let url = uvr::fetch::packages_url(&rest[1]);
    match uvr::fetch::get_text(&url) { Ok(t) => (t, &rest[2..]), Err(e) => { ...; return FAILURE } }
}
```
`cargo test` → ✅ 24 passed。**真·联网端到端**：
```
$ cargo run -- lock --repo https://jeroen.r-universe.dev jsonlite
# uvr lockfile v1
jsonlite 2.0.1
```
（`jsonlite` 依赖 `methods`，被识别为自带、跳过。）

## 4. 改了哪些文件 / 加了什么
- `src/resolver.rs`：新增 `BUILTIN` 常量集 + `is_builtin`；求解跳过自带包；新增 `skips_builtin_packages` 测试。
- `src/main.rs`：`lock` 增加 `--repo <url>` 分支——联网抓 `PACKAGES` 再求解；保留本地文件用法。

## 5. 学到的语法 / 技巧
- **`const BUILTIN: &[&str] = &[...]`**：编译期常量的字符串切片数组。
- **`slice.contains(&name)`**：判断元素是否在切片中。
- **`match rest.first().map(String::as_str) { Some("--repo") if ... => ... }`**：对第一个参数分支，区分"联网"与"本地文件"两种数据来源。
- **切片 `&rest[2..]` / `&rest[1..]`**：取剩余参数作为根包列表。

## 6. 语言设计巧思
- **把"环境已知事实"编码进代码**：哪些包随 R 发行，是 R 生态的客观事实；用一个常量集合显式表达，比散落各处的特判更清晰、可维护。
- **一个核心、两个数据源**：本地文件与联网抓取最终都汇成"`PACKAGES` 文本 → `lock_from_packages`"。核心求解逻辑不关心数据从哪来——适配器把差异挡在外面。

## 7. 领域知识
- **base / recommended 包**：R 自带约 30 个包（base 14 个 + recommended 15 个），它们不出现在仓库 `PACKAGES` 里、也无需单独安装。任何 R 包管理器都必须知道这份名单，否则会去"安装"根本装不了的东西。
- **跨仓库依赖（本步未解决）**：r-universe / drat 等小仓库里的包，常依赖 CRAN 上的包。单一 `PACKAGES` 解不全时会 `NotFound`；真实工具会**合并多个仓库**的索引——这是后续可扩展点。

## 8. 软件设计理念
- **领域知识显式化**：把隐性约定（自带包名单）变成代码里可见、可测的常量。
- **适配器复用**：新增一个数据来源（网络）只动 `main` 的入口分支，核心逻辑零改动——这正是 Step 16/17"薄壳 + 防腐层"设计的回报。

## 9. 小测验（自测）
1. 不跳过 `utils`/`methods` 会发生什么？为什么？
2. `BUILTIN` 为什么用 `&[&str]` 常量，而不是每次构造一个集合？
3. CLI 的 `--repo` 与本地文件两条路，最终汇到哪个函数？这说明了什么设计？
4. 为什么 r-universe 上的包用 `--repo` 求解仍可能 `NotFound`？

## 10. 参考答案
1. 它们不在 `PACKAGES` 里，求解会判定"找不到包"返回 `Err(NotFound(...))`；因为它们随 R 自带、本就不该去仓库找。
2. 它是编译期就固定的只读数据，用常量切片零分配、可被 `is_builtin` 直接查；无需每次重建集合。
3. 都汇到 `commands::lock_from_packages`（核心求解 + 渲染）。说明数据来源被隔离在适配器层，核心逻辑与 IO 解耦。
4. 因为小仓库的包常依赖 CRAN 上、不在该仓库 `PACKAGES` 里的包；单仓库索引不全，故 `NotFound`。需合并多仓库索引才能解全。

## 11. 下一步预告
模块 C（联网）收官：`uvr` 已能从真实仓库联网抓取并求解。接着开 PR 合入 `main`。
之后做 **模块 E：下载 + 安装**——把解出的包真的下载下来、用 `R CMD INSTALL` 装进**项目本地 R 库**（不碰你的全局 R 环境）。
