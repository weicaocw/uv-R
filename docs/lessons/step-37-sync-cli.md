# Step 37：命令行 `uvr sync`——按 lockfile 一键还原环境

> 模块：O sync ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-37-sync-cli.md`）｜ 产物：`src/main.rs`（`sync` + `resolve_r_bin` 助手）｜ 上一步：[Step 36](step-36-sync-plan.md)

## 0. 一句话目标
把 Step 36 的同步逻辑接到终端：`uvr sync --repo <url> [--lib <dir>] [<lockfile>]`，默认读 `uvr.lock`。

## 1. 前置回顾
Step 36 在库里做好了 `sync_from_lock`（解析 lockfile → 不求解 → 下载装）。本步是集成：CLI 读文件、抓仓库（暖缓存）、解析该用的 R，然后调它。

## 2. "测试"：端到端演示（show, don't tell）
本机真跑，把 `lock` 和 `sync` 串起来：
```
$ uvr lock --repo <r-universe> dotenv > uvr.lock
$ cat uvr.lock
# uvr lockfile v1
dotenv 1.0.3.9000

$ uvr sync --repo <r-universe> --lib ./r-lib
→ 使用 R / using R 4.5.2: /usr/local/bin/R
synced dotenv 1.0.3.9000
→ 已按 uvr.lock 还原到项目本地库 / restored from uvr.lock into: ./r-lib

$ grep ^Version ./r-lib/dotenv/DESCRIPTION
Version: 1.0.3.9000        # ← 正是锁定的版本
```
CLI 分发逻辑由集成演示覆盖；纯逻辑（`sync_plan`）已在 Step 36 单测。全量 55 测试（+3 ignored）绿。

## 3. 实现到通过（TDD 绿）
- `main()` 加分支 `Some("sync") => sync(&args[2..])`，`usage()` 加一行。
- `sync(rest)`：复用 `parse_flags` 拿 `--repo`/`--lib`/位置参数；位置参数第一个当 lockfile 路径，没有就默认 `uvr.lock`。读文件 → `fetch_sources`（暖缓存）→ `resolve_r_bin` → `sync_from_lock`。
- **DRY**：把 `install` 里"解析 R 并打印用哪个"的那段抽成 `resolve_r_bin() -> Option<PathBuf>`，`install` 与 `sync` 共用——保证两者"用哪个 R"的行为**完全一致**。

## 4. 改了哪些文件 / 加了什么
- `src/main.rs`：`sync` 处理函数、`resolve_r_bin` 助手、`main` 分发 + `usage` 更新；`install` 改用助手。

## 5. 学到的语法 / 技巧
- **`let Some(r_bin) = resolve_r_bin() else { return ...; }`**：let-else 再次登场——拿不到 R 就直接退出，主流程不缩进。
- **位置参数兜默认**：`positional.first().map(String::as_str).unwrap_or("uvr.lock")`——没给 lockfile 就用约定的默认文件名。和 `git`（默认当前分支）、`cargo`（默认 `Cargo.toml`）一样的"约定优于配置"。
- **复用 `parse_flags`**：同一个参数解析器服务 `install` 和 `sync`（`--repo`/`--lib` 语义一致），位置参数各自解释（`install` 当包名，`sync` 当 lockfile 路径）。一套解析、多处复用。
- **`Option<PathBuf>` 表达"成功并打印 / 失败已报错"**：`resolve_r_bin` 把"解析 + 打印 + 错误处理"封进一个返回 `Option` 的助手，调用方只需 `Some/None` 二选一。

## 6. 设计巧思 / 方法论
- **命令家族的一致性**：`install` 和 `sync` 形状几乎一样（抓仓库 → 选 R → 下载装），区别只在"装什么"（求解 vs lockfile）。让它们**长得像**、共享助手（`parse_flags`/`fetch_sources`/`resolve_r_bin`/`run_plan`），用户学一个就会另一个，代码也不重复。
- **约定的默认值降低摩擦**：`uvr.lock` 作为默认 lockfile，让最常见用法 `uvr sync --repo ...` 一句话搞定，不必每次写文件名。

## 7. 领域知识（R / 包管理）
- **典型工作流**：`uvr lock --repo ... pkg... > uvr.lock`（提交进库）→ 队友 / CI / 新机器上 `uvr sync --repo ...` 还原出**一模一样**的依赖。这正是 `renv::snapshot()` + `renv::restore()` 的对应物，也是 `npm ci`、`uv sync` 的心智模型。
- **为什么 sync 仍要 `--repo`**：lockfile v1 不记来源仓库，故下载来源由 `--repo` 提供（见 Step 36）。

## 8. 软件设计理念
- **CLI 命令是组合出来的**：`sync` 不是从零写的——它把已有的积木（`parse_flags`、`fetch_sources`、`resolve_r_bin`、`sync_from_lock`）拼起来。一个设计良好的系统，**新命令往往是旧零件的新排列**。这就是为什么前面坚持把逻辑做成小而纯的可复用件。

## 9. 小测验（自测）
1. `uvr sync` 和 `uvr install` 在"装什么"上有何根本区别？在 CLI 实现上又共享了哪些零件？
2. 为什么 `sync` 把 lockfile 路径设计成"可省略、默认 `uvr.lock`"？
3. 把"解析该用哪个 R"抽成 `resolve_r_bin` 共用，避免了什么潜在 bug？
4. 用一句话描述"`lock` 一次、到处 `sync`"为什么能保证可复现。

## 10. 参考答案
1. **装什么**：`install` 现求解（可能升级），`sync` 按 lockfile 装锁定版本（不漂移）。**共享零件**：`parse_flags`（解析 `--repo`/`--lib`）、`fetch_sources`（暖缓存抓仓库）、`resolve_r_bin`（选 R）、`run_plan`（下载 + 安装）。
2. 降低摩擦：最常见的用法（项目根有 `uvr.lock`）一句 `uvr sync --repo ...` 即可，不必每次敲文件名。需要时仍可显式传别的 lockfile 路径。
3. 避免 `install` 和 `sync` 各写一份 R 解析逻辑、日后改了一处忘了另一处，导致"两个命令用不同的 R"这种隐蔽不一致。抽成一个助手，行为天然统一。
4. 因为 `lock` 把确切版本写进文件、`sync` 严格照装不再求解——同一个 lockfile 在任何机器 / 任何时间都还原出相同版本。

## 11. 下一步预告
模块 O 完成、发布 **v0.8**。后续可继续：并行下载 / 安装（模块 K，提升装大量包的吞吐）、lockfile v2（记录每个包的来源仓库，让 `sync` 自包含）。
