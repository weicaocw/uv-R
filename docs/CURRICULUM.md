# uvr 课程地图（CURRICULUM）

> 方法：自顶向下**理解**全貌，自底向上**建造**。教学课在 `docs/lessons/`，每步一篇；提交 / PR 中英双语。

## 整体目标
用 Rust 造一个类似 uv / Cargo 的 R 语言包管理器 `uvr`。沿途系统学习 **Rust、R 包管理、软件设计**。

## 模块总览（七章全部完成 🎉）
| 章 | 模块 | 内容 | 状态 |
|---|---|---|---|
| A | 版本模型 | 解析、比较、约束 | ✅（PR #1） |
| B | 元数据 | DCF 解析、依赖字段、依赖图 | ✅（PR #2） |
| C | 联网 | HTTP 抓取 PACKAGES + 跳过自带包 + `--repo` | ✅（PR #5） |
| D | 依赖求解 | 选版本、传递依赖、冲突、lockfile | ✅（PR #3，手写教学版） |
| E | 下载 + 安装 | 下载 tarball + `R CMD INSTALL` 到项目本地库 | ✅（PR #6） |
| F | 命令行 | `uvr lock` / `uvr install` | ✅ → v0.1 |
| G | Benchmark | 对 pak 的诚实对照（自写计时） | ✅ → **v0.2** |

uvr 现已具备：联网抓取 → 依赖求解 → lockfile → 项目本地安装 → 对 pak 的 benchmark，全程测试覆盖、CI 绿。

## 守住的底线
- E 安装到**项目本地 R 库**（`-l` 隔离，未碰全局 R 环境）。
- G **自写计时脚本**（绕开需 brew 的 hyperfine）；只比能公平比的轴，安装这类物理下限诚实报"打平"。

## 各模块小步索引（详见 docs/lessons/，共 21 课）
- **A 版本模型** — 01 骨架 · 02 Version · 03 解析 · 04 比较 · 05 修正(Eq/Ord) · 06 约束 · 07 CI。
- **B 元数据** — 08 拆库+模块 · 09 DCF · 10 依赖字段 · 11 包索引。
- **D 依赖求解** — 12 best_match · 13 传递依赖 · 14 冲突检测 · 15 lockfile。
- **F 命令行** — 16 `uvr lock`。
- **C 联网** — 17 ureq 抓取层 · 18 跳过自带包 + `--repo`。
- **E 下载+安装** — 19 下载 + `R CMD INSTALL` · 20 `uvr install`。
- **G Benchmark** — 21 自写计时，对 pak 诚实对照（见 `BENCHMARK.md`）。

## 可能的后续（v0.3+）
- 把手写教学求解器升级为工业级 `pubgrub`（带回溯）。
- 合并多仓库索引（解决跨仓库依赖 `NotFound`）。
- 元数据 / 下载缓存；binary 包优先；并行下载安装。
