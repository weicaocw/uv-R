# uvr 课程地图（CURRICULUM）

> 方法：自顶向下**理解**全貌，自底向上**建造**。每完成一步就更新本表的进度标记。
> 教学课在 `docs/lessons/`，每步一篇；提交 / PR 中英双语。

## 整体目标
用 Rust 造一个类似 uv / Cargo 的 R 语言包管理器 `uvr`。沿途系统学习 **Rust、R 包管理、软件设计**。

## 模块总览
| 章 | 模块 | 内容 | 状态 |
|---|---|---|---|
| A | 版本模型 | 解析、比较、约束 | ✅ 完成（PR #1） |
| B | 元数据 | DCF 解析、依赖字段、依赖图 | ✅ 完成（PR #2） |
| D | 依赖求解 | 选版本、传递依赖、冲突、lockfile | ✅ 完成（PR #3，手写教学版） |
| F | 命令行 | `uvr lock`（纯 std） | ✅ 完成 → **v0.1** |
| C | 联网 | HTTP 抓取 PACKAGES + 跳过自带包 + `--repo` | ✅ 完成 |
| **E** | **下载 + 安装** ← 当前 | 下载 tarball / `R CMD INSTALL`（项目本地库） | 进行中 |
| G | Benchmark | 对 pak 的对照实验（自写计时） | ⬜ 计划 |

> 底线：E 安装到**项目本地 R 库**（不污染全局 R 环境）；G **自写计时脚本**（绕开需 brew 的 hyperfine）。

## 各模块小步索引（详见 docs/lessons/）
- **A 版本模型** ✅ — 01 骨架 · 02 Version · 03 解析 · 04 比较(derive) · 05 修正(Eq/Ord 契约) · 06 约束 · 07 CI。
- **B 元数据** ✅ — 08 拆库+模块 · 09 DCF · 10 依赖字段 · 11 包索引/依赖图。
- **D 依赖求解** ✅ — 12 best_match(生命周期) · 13 传递依赖 · 14 冲突检测 · 15 lockfile。
- **F 命令行** ✅ — 16 `uvr lock`（env::args + ExitCode）。
- **C 联网** ✅ — 17 ureq 抓取层 · 18 跳过自带包 + `uvr lock --repo <url>` 联网求解。
- **E 下载+安装** ← 当前 — 下载 tarball、`R CMD INSTALL` 到项目本地库。
- **G Benchmark**（计划）— 自写计时，对 `pak` 跑对照，出报告。
