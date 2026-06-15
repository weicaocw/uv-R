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
| C | 联网 + 缓存 | HTTP 抓取 PACKAGES、缓存 | ⏸ 资源墙（需实时网络 / 外部 crate） |
| D | 依赖求解 | 选版本、传递依赖、冲突、lockfile | ✅ 完成（PR #3，手写教学版） |
| E | 下载 + 安装 | binary 解压 / `R CMD INSTALL` | ⏸ 资源墙（需网络 / R） |
| F | 命令行 | `uvr lock`（纯 std 最小实现） | ✅ 完成 → **v0.1** |
| G | Benchmark + 报告 | 对 pak 的对照实验 | ⏸ 资源墙（需 R / pak / hyperfine） |

> **v0.1（可发布成品）**：模块 A + B + D + F = 一个**离线依赖求解器**——
> `uvr lock <PACKAGES 文件> <根包>...` 读取仓库索引、解出传递依赖、写出 lockfile，全程离线、全测试覆盖、CI 绿。
> 需实时网络 / R / 系统安装的模块（C、E、G）留作资源墙，待具备条件再做。

## 各模块小步索引（详见 docs/lessons/）
- **模块 A：版本模型** ✅ — 01 骨架 · 02 Version 结构体 · 03 解析 · 04 比较(derive) · 05 修正比较(Eq/Ord 契约) · 06 版本约束 · 07 CI。
- **模块 B：元数据** ✅ — 08 拆库 + 模块系统 · 09 DCF 解析 · 10 依赖字段解析 · 11 包索引 / 依赖图。
- **模块 D：依赖求解** ✅（手写教学版，非 pubgrub）— 12 best_match(生命周期) · 13 传递依赖(工作队列) · 14 冲突检测(Result/错误枚举) · 15 lockfile(Display + 往返)。
- **模块 F：命令行** ✅ — 16 `uvr lock`（env::args + 纯函数核心 + ExitCode）。

## 资源墙（交还给用户处理）
- **C 联网**：抓 CRAN/PPM 的 `PACKAGES` 需实时网络；引入 `reqwest`/`ureq` 需 cargo 联网拉 crate。
- **E 安装**：解压 binary 包 / 调 `R CMD INSTALL` 需 R 与系统工具链。
- **G Benchmark**：对 `pak` 跑对照需 R + pak + `hyperfine`（需 `brew install`）。
- **D 升级**：把教学版求解器换成 `pubgrub`（需 cargo 联网拉 crate）。
