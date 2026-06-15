# Step 25：CLI 多 `--repo`（跨仓库求解与安装）

> 模块：H 多仓库 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-25-cli-multi-repo.md`）｜ 测试：✅ 31 passed + 真·跨仓库 demo ｜ 上一步：[Step 24](step-24-multi-repo.md)

## 0. 一句话目标
让 `uvr lock` / `uvr install` 接受**多个 `--repo`**：抓取并合并多个仓库后求解；安装时按**每个包自己的仓库**下载。

## 1. 前置回顾
Step 24 让索引能合并、包能记住来源。本步把它接到命令行，并用一个**真实的跨仓库依赖**验证。

## 2. 先写测试（TDD·红）
```rust
#[test]
fn install_plan_uses_each_packages_own_repo() {
    // pkgX 在 r1、依赖 pkgY；pkgY 在 r2。各自地址应指向各自仓库。
    let sources = vec![
        ("Package: pkgX\nVersion: 1.0\nImports: pkgY\n".into(), "https://r1".into()),
        ("Package: pkgY\nVersion: 2.0\n".into(), "https://r2".into()),
    ];
    let plan = install_plan(&sources, &["pkgX".to_string()]).unwrap();
    let y = plan.iter().find(|i| i.name == "pkgY").unwrap();
    assert_eq!(y.url, "https://r2/src/contrib/pkgY_2.0.tar.gz"); // 指向 r2，而非 pkgX 的 r1
}
```

## 3. 实现到通过（TDD·绿）
- `commands` 重构为**多源**：`type Source = (String, String)`（PACKAGES 文本 + 仓库基址）；`build_index` 合并；`lock_from_sources` / `install_plan(sources, roots)`；`install_plan` 用**每个包自己的 `repo`** 拼下载地址。
- `main` 加 `parse_flags`（收集多个 `--repo` 与 `--lib`）与 `fetch_sources`（抓取每个仓库）。

`cargo test` → ✅ 31 passed。**真·跨仓库**：
```
# 单仓库：franc 依赖 jsonlite 不在本仓 → 解不出
$ uvr lock --repo https://gaborcsardi.r-universe.dev franc
求解失败: NoSolution(... NoVersions("jsonlite") ...)

# 多仓库：jsonlite 在 jeroen 的仓 → 跨仓库解出
$ uvr lock --repo https://gaborcsardi.r-universe.dev --repo https://jeroen.r-universe.dev franc
# uvr lockfile v1
franc 1.1.4.9000
jsonlite 2.0.1
```

## 4. 改了哪些文件 / 加了什么
- `src/commands.rs`：改为多源接口（`Source`、`lock_from_sources`、`install_plan(sources, roots)`、`install_packages(sources, …)`）；`install_plan` 按包来源拼 URL；新增"按各自仓库"测试。
- `src/main.rs`：`parse_flags` 解析多个 `--repo`/`--lib`；`fetch_sources` 抓取并组装；`lock` 兼容本地文件用法。

## 5. 学到的语法 / 技巧
- **类型别名 `type Source = (String, String)`**：给"语义二元组"起个名字，签名更可读。
- **手写多标志解析**：`while i < rest.len() { match rest[i] { "--repo" => 累积, … } }`，把可重复的 `--repo` 收进 `Vec`。
- **在闭包里回查来源**：`index.versions_of(&name).iter().find(|p| p.version == version).map(|p| p.repo.as_str())`——拿到该包的仓库。

## 6. 语言设计巧思
- **纯函数吃"已抓好的数据"**：`lock_from_sources` / `install_plan` 只接受 `&[Source]`（文本已在手），不触网——于是可在无网络下单测整条"合并→求解→计划"逻辑。网络只在 `main` 的 `fetch_sources` 一处。
- **provenance 落地**：Step 24 让包带上 `repo`，本步在"拼下载地址"时用上——信息在最早处记录、在最晚处使用，中间一路携带。

## 7. 领域知识
- **跨仓库依赖**在 R 生态很常见（r-universe 的包依赖 CRAN 的包）。要解全，必须合并多仓库索引。
- **pubgrub 的无解解释**：`NoVersions("jsonlite")` 清楚指出"jsonlite 没有可用版本"——回溯求解器能告诉你**为什么**解不出。
- 这一步让 uvr 的求解贴近真实世界，是"能与 pak 公平对比"的必要能力。

## 8. 软件设计理念
- **数据流清晰**：`sources → 合并索引 → 求解 → 安装计划 → 下载安装`，每段职责单一、可单测。
- **接口随能力演进、核心稳定**：从"单仓库"到"多仓库"，对外签名变了（`Source` 列表），但求解核心 `resolve_pubgrub` 一字未动——分层把变化挡在边缘。

## 9. 小测验（自测）
1. 为什么 `install_plan` 要按"每个包自己的仓库"而不是单一仓库拼下载地址？
2. `lock_from_sources` 为什么不直接联网，而是接受已抓好的 `Source` 列表？
3. 多个 `--repo` 是怎么被解析收集的？
4. 单仓库解 `franc` 时，pubgrub 报的 `NoVersions("jsonlite")` 说明了什么？

## 10. 参考答案
1. 多仓库下，不同包来自不同仓库，下载地址各不相同；用每个包自己的 `repo` 才能正确定位 tarball。
2. 为了让核心逻辑**可离线单测**：把网络隔离在 `main`，纯函数只处理数据；也便于将来加缓存（数据可来自缓存而非网络）。
3. `parse_flags` 在循环里遇到 `--repo` 就把后一个参数 push 进 `repos` 向量，可重复多次。
4. 说明在已合并的索引里，`jsonlite` 没有任何版本可选（它不在 gaborcsardi 仓库）；加入含 jsonlite 的仓库后即可解出。

## 11. 下一步预告
模块 H（多仓库）收官，开 PR 合入 `main`（v0.4）。
下一模块 **I：缓存**——把抓到的 `PACKAGES` 与下载的 tarball 缓存到本地，重复命令走"暖缓存"。这正是 uv 招牌 benchmark 的关键轴，也是让 uvr **有底气与 pak 做端到端对比**的下一块拼图。
