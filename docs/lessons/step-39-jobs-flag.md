# Step 39：给并行加旋钮——`--jobs <N>`

> 模块：K 并行 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-39-jobs-flag.md`）｜ 产物：`src/commands.rs`（`run_plan`/`install_packages`/`sync_from_lock` 增 `jobs` 参数）+ `src/main.rs`（`parse_flags` 解析 `--jobs`）｜ 上一步：[Step 38](step-38-parallel-download.md)

## 0. 一句话目标
让用户能控制并行下载的并发度：`--jobs <N>`，默认 = CPU 核数。

## 1. 前置回顾
Step 38 让下载并行了，但并发度写死在 `run_plan` 里（`available_parallelism`）。把它**参数化**、暴露成命令行旋钮，用户就能按网络 / 机器情况调（慢网调小、强机调大）。

## 2. "测试"：演示 + 既有单测护栏
`--jobs` 是把一个 `usize` 一路传下去，逻辑上由 Step 38 的并行测试覆盖（`parallel_for_each` 接受任意并发度）。本机演示：
```
$ uvr install --repo <r-universe> dotenv --lib /tmp/... --jobs 4
→ 使用 R / using R 4.5.2: /usr/local/bin/R
installed dotenv 1.0.3.9000
```
全量 59 测试（+3 ignored）绿——证明加参数没破坏既有行为。

## 3. 实现到通过（TDD 绿）
- **贯通 `jobs`**：`run_plan(plan, lib, dl, r_bin, jobs)` 不再自己算并发度，改为接收；`install_packages` / `sync_from_lock` 加 `jobs` 参数透传。
- **CLI 解析**：`parse_flags` 增 `--jobs <N>` 分支，默认 `default_jobs()`（= `available_parallelism`）；非法 / 0 退回默认并 `max(1)`。返回元组多一个 `usize`。
- **更新调用点**：`lock`（忽略 `jobs`，用 `_jobs`）、`install`、`sync` 三处解构 + 传递。

## 4. 改了哪些文件 / 加了什么
- `src/commands.rs`：`run_plan` / `install_packages` / `sync_from_lock` 增 `jobs`；更新一个测试调用。
- `src/main.rs`：`default_jobs`、`parse_flags` 解析 `--jobs`、三个调用点解构、`usage` 与模块文档更新。

## 5. 学到的语法 / 技巧
- **参数化已有默认值**：把硬编码的策略（并发度）抽成参数，默认值仍由 `default_jobs()` 提供——**默认合理、可被覆盖**。好的 CLI 都是这样：零配置能用，需要时能调。
- **`parse().unwrap_or(jobs).max(1)`**：解析用户输入，失败就退回默认，再保证至少 1。**永不因坏输入崩溃**，也永不开 0 个线程。容错的参数解析。
- **元组返回值的演化**：`parse_flags` 的返回从三元组变四元组。Rust 的解构赋值让调用点改动一目了然（`let (repos, lib, jobs, roots) = ...`）；不关心的位置用 `_jobs` 显式忽略，编译器不报未用警告。
- **`available_parallelism()`**：标准库探测"可用并行度"（通常 = 逻辑核数），返回 `Result`（容器 / 沙箱里可能取不到），故 `.map(...).unwrap_or(4)` 兜底。

## 6. 设计巧思 / 方法论
- **机制与策略分离**：`parallel_for_each`（机制：怎么并行）不关心并发度从哪来；`run_plan` 接收 `jobs`（策略：并行多少）；CLI 决定默认与覆盖。每层只管自己那块，旋钮接在最外层。
- **合理默认 > 强制配置**：默认 = CPU 核数，对绝大多数人开箱即最优；`--jobs` 留给少数要调优的场景。把常见情形做到零摩擦，是好工具的标志。

## 7. 领域知识（性能调优）
- **并发度怎么选**：下载是 IO 密集，并发度常可**高于**核数（线程多在等网络）。但太高会拖垮慢网 / 触发服务端限流。默认取核数是稳妥起点；用户可据带宽与对端策略用 `--jobs` 调整。
- **和 uv 一致的旋钮**：uv 等现代工具都提供并发度控制（如 `UV_CONCURRENT_DOWNLOADS`）。给出旋钮，让工具适配从笔记本到 CI 的各种环境。

## 8. 软件设计理念
- **可配置性要克制**：只暴露**真正需要**的旋钮（这里就一个 `--jobs`），且每个都有合理默认。旋钮太多会压垮用户；太少又不够灵活。"默认能用、关键可调"是平衡点。

## 9. 小测验（自测）
1. 为什么默认并发度取 `available_parallelism()` 而不是一个写死的常数（如 8）？
2. `parse().unwrap_or(jobs).max(1)` 三步各防住什么？
3. "机制与策略分离"在本步体现在哪三层？
4. 下载并发度为什么常常可以、甚至应该高于 CPU 核数？

## 10. 参考答案
1. 不同机器核数不同；`available_parallelism()` 自适应（4 核取 4、16 核取 16），开箱即接近最优。写死常数要么在小机上过载、要么在大机上浪费并行度。
2. `parse()`：把字符串转 `usize`，可能失败；`unwrap_or(jobs)`：失败就退回默认值（不崩）；`max(1)`：保证至少 1 个线程（防用户传 0 导致一个都不跑）。
3. **机制**：`parallel_for_each`（怎么并行）；**策略量**：`run_plan` 接收的 `jobs`（并行多少）；**策略来源**：CLI 的 `--jobs` 与 `default_jobs()`（默认与覆盖）。三层各司其职。
4. 下载多在**等网络**（IO 阻塞），线程等待时不占 CPU，所以并发数高于核数仍能提升吞吐（更多请求同时在途）。但要权衡慢网带宽与对端限流，故留 `--jobs` 可调。

## 11. 下一步预告
模块 K 完成、发布 **v0.9**。后续可继续：**lockfile v2**（记录每个包来源仓库，让 `sync` 自包含、免 `--repo`）；**拓扑序安装**（依赖先于依赖者，为"多包并行安装"铺路）；以及换到支持 binary 的环境补齐模块 J。
