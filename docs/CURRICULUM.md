# uvr 课程地图（CURRICULUM）

> 方法：自顶向下**理解**全貌，自底向上**建造**。教学课在 `docs/lessons/`，每步一篇；提交 / PR 中英双语。

## 整体目标
用 Rust 造一个类似 uv / Cargo 的 R 语言包管理器 `uvr`。沿途系统学习 **Rust、R 包管理、软件设计**。

## 模块总览（全部完成 🎉）
| 章 | 模块 | 内容 | 状态 |
|---|---|---|---|
| A | 版本模型 | 解析、比较、约束 | ✅（PR #1） |
| B | 元数据 | DCF 解析、依赖字段、依赖图 | ✅（PR #2） |
| C | 联网 | HTTP 抓取 PACKAGES + 跳过自带包 + `--repo` | ✅（PR #5） |
| D | 依赖求解（手写教学版） | 选版本、传递依赖、冲突、lockfile | ✅（PR #3） |
| E | 下载 + 安装 | 下载 tarball + `R CMD INSTALL` 到项目本地库 | ✅（PR #6） |
| F | 命令行 | `uvr lock` / `uvr install` | ✅ → v0.1 |
| G | Benchmark | 对 pak 的诚实对照（自写计时） | ✅ → v0.2 |
| D′ | 升级 pubgrub | 工业级回溯求解器，默认启用 | ✅ → **v0.3** |

uvr 能力：联网抓取 → **pubgrub 回溯求解** → lockfile → 项目本地安装 → 对 pak benchmark。手写贪心求解器保留作对照教学。

## 各模块小步索引（详见 docs/lessons/，共 23 课）
- **A 版本模型** — 01–07（骨架/Version/解析/比较/Eq-Ord 契约/约束/CI）。
- **B 元数据** — 08–11（拆库+模块/DCF/依赖字段/包索引）。
- **D 依赖求解（教学版）** — 12–15（best_match/传递依赖/冲突检测/lockfile）。
- **F 命令行** — 16（`uvr lock`）。
- **C 联网** — 17–18（ureq 抓取/跳过自带包+`--repo`）。
- **E 下载+安装** — 19–20（下载+`R CMD INSTALL`/`uvr install`）。
- **G Benchmark** — 21（自写计时，对 pak 诚实对照）。
- **D′ pubgrub** — 22–23（OfflineDependencyProvider 适配器 + 回溯/对照测试；CLI 切到 pubgrub）。

## 守住的底线
- E 安装到**项目本地 R 库**（`-l` 隔离，未碰全局 R 环境）。
- G **自写计时脚本**（不装 hyperfine）；只比能公平比的轴，安装诚实报"打平"。
