# Step 36：按 lockfile 同步——`sync_plan`（不求解、防漂移）

> 模块：O sync ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-36-sync-plan.md`）｜ 产物：`src/commands.rs`（`run_plan` 重构 + `sync_plan` + `sync_from_lock`）｜ 上一步：[Step 35](step-35-user-manual.md)

## 0. 一句话目标
做出 `sync` 的纯逻辑核心：给定 lockfile 里锁定的版本，**不重新求解**，严格按锁定版本拼出下载安装计划。

## 1. 前置回顾
- `uvr lock` 会把求解结果写成 lockfile（`name version` 一行一个）。
- `uvr install` 每次都**重新求解**，可能装到比上次更高的版本。

但"可复现"要的恰恰相反：换台机器、过段时间，要装出**一模一样**的版本。这就是 `sync`——读 lockfile，**照着装**，绝不漂移。对标 `uv sync` / `cargo build`（按 `Cargo.lock`）。

## 2. "测试"先行（TDD 红）
最关键的一条测试，钉死"不漂移"：
```rust
// 索引里同时有 pkgA 1.0.0 和 1.2.0；锁定 1.0.0
sync_plan({pkgA: 1.0.0}, sources) -> 计划里 pkgA 用 1.0.0（不是 1.2.0）
```
再加两条边界：
```rust
sync_plan({pkgA: 9.9.9}, sources)            -> Err 含 "pkgA"   // 锁定版本仓库里没有
sync_from_lock("乱码", ...)                   -> Err 含 "lockfile" // lockfile 解析失败
```

## 3. 实现到通过（TDD 绿）
```rust
pub fn sync_plan(locked: &BTreeMap<String, Version>, sources: &[Source])
    -> Result<Vec<InstallItem>, String>
{
    let index = build_index(sources);
    locked.iter().map(|(name, version)| {
        let found = index.versions_of(name).iter()
            .find(|p| p.version == *version)               // 精确匹配锁定版本
            .ok_or_else(|| format!("锁定的 {name} {version} 在仓库里找不到 / ... not found"))?;
        let url = install::tarball_url(&found.repo, name, &version.to_string());
        Ok(InstallItem { name: name.clone(), version: version.to_string(), url })
    }).collect()                                            // Vec<Result> → Result<Vec>
}
```
外加 `sync_from_lock(text, ...)` = `lockfile::parse` + `sync_plan` + 下载安装。

**顺手重构（DRY）**：`install` 和 `sync` 都要"逐个下载 + `R CMD INSTALL`"。把这段抽成私有的 `run_plan(plan, lib, dl, r_bin)`，两边都调它。7 个 commands 测试全绿。

## 4. 改了哪些文件 / 加了什么
- `src/commands.rs`：抽出 `run_plan`；`install_packages` 改用它；新增 `sync_plan` + `sync_from_lock` + 3 个测试。

## 5. 学到的语法 / 技巧
- **`Vec<Result<T,E>>` → `Result<Vec<T>,E>`**：对迭代器 `.map(|..| -> Result).collect()`，Rust 能把"一串可能失败的结果"收集成"整体的成败"——**任一失败就短路返回那个 Err**。这是处理"批量、可失败"的惯用法。
- **闭包里用 `?`**：`map` 的闭包返回 `Result`，于是闭包内能用 `?` 提前抛错（抛出的 Err 成为该元素的 `Err`，再被 `collect` 短路）。
- **`find(|p| p.version == *version)`**：`version` 是 `&Version`（来自 `&BTreeMap` 遍历），比较时解引用一次。
- **重构而非复制**：发现 `install` 与 `sync` 的尾巴一样 → 抽 `run_plan`。一处实现、两处复用，将来改安装逻辑只改一处。这是"第三次出现就抽象"的工程直觉（这里是第二次，但抽象边界很清晰，值得）。

## 6. 设计巧思 / 方法论
- **lock 与 sync 的分工**：`lock`=**决策**（求解出该用哪些版本，写进文件）；`sync`=**执行**（照文件装，不再决策）。把"想清楚"和"照着做"分开，是可复现系统的核心——`cargo`、`uv`、`npm ci` 都是这个套路。
- **防漂移是正确性，不是优化**：`sync_plan` 精确匹配锁定版本，哪怕仓库里有更新。若它"顺手升级"，就破坏了 lockfile 的全部意义。**严格 = 可信**。
- **错误指名道姓**：锁定版本找不到时，报错带上是哪个包哪个版本——用户能立刻定位（多半是仓库下架了旧版，或换了仓库）。

## 7. 领域知识（R / 包管理）
- **为什么 R 项目需要 lockfile/sync**：R 社区的 `renv` 正是干这个——把项目依赖的确切版本锁住、可在另一台机器还原。CRAN 只保留每个包的**最新版**，旧版进 Archive；所以"锁定 + 能从某处取到那个版本"对可复现至关重要（也是为什么 `sync` 需要 `--repo` 指明取处）。
- **lockfile v1 的局限**：目前只记 `name version`，不记仓库；故 `sync` 仍需 `--repo`。未来可升级 lockfile v2 记录每个包的来源仓库，让 `sync` 自包含。

## 8. 软件设计理念
- **纯核心 + 薄壳，再来一次**：`sync_plan` 是纯函数（给 locked + sources，出计划或错误），可被穷举单测；真正的下载安装在 `run_plan` 这层薄壳。整条 `install`/`sync` 都遵循这个结构——一致的架构让新功能"长"得很自然。

## 9. 小测验（自测）
1. `sync` 和 `install` 最本质的区别是什么？为什么 `sync` 不能"顺手升级"到更高版本？
2. `.map(...).collect::<Result<Vec<_>,_>>()` 在其中一个元素失败时会怎样？
3. 为什么把下载安装循环抽成 `run_plan`？带来什么好处？
4. 为什么 `sync` 目前还需要 `--repo`，而不能只靠 lockfile？

## 10. 参考答案
1. `install` 每次**重新求解**（可能升级）；`sync` 严格按 lockfile **照装**（锁定版本）。`sync` 的全部价值是**可复现**——若顺手升级，换台机器/换时间就装出不同版本，lockfile 形同虚设。
2. `collect` 成 `Result<Vec<_>, _>` 时**短路**：遇到第一个 `Err` 立即返回该 `Err`，不再处理后续元素；全 `Ok` 才得到 `Ok(Vec)`。
3. `install` 与 `sync` 的"下载 + 安装"循环完全相同。抽成 `run_plan` 后一处实现、两处复用：改安装行为（如加重试、加日志）只改一处，不会两边不一致。
4. lockfile v1 只记了 `name version`，没记每个包来自哪个仓库；`sync` 得知道去哪下载，故仍需 `--repo`。升级到记录来源的 lockfile v2 后即可自包含。

## 11. 下一步预告
Step 37：把 `sync` 接到命令行——`uvr sync --repo <url> [--lib <dir>] [<lockfile>]`（默认读 `uvr.lock`），并抽一个 `resolve_r_bin` 助手给 `install`/`sync` 共用。然后端到端演示 lock → sync 还原环境。
