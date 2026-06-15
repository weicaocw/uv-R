# Step 30：发现系统里所有的 R

> 模块：M R 版本管理 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-30-discover-r.md`）｜ 产物：`src/rversion.rs`（`RInstall` / `r_candidate_paths_from` / `dedup_installs` / `probe_r` / `discover`）｜ 上一步：[Step 29](step-29-parse-r-version.md)

## 0. 一句话目标
枚举本机所有可能的 R，逐个问版本，得到一张去重后的"**路径 → 版本**"清单。

## 1. 前置回顾
Step 29 我们能从一段文本里抠出版本号了。但文本从哪来？得先**找到** R 的可执行文件，再运行 `R --version` 拿到那段文本。本步把"找 + 问 + 去重"串起来，做出 `discover()`。

## 2. "测试"先行（TDD 红）
把逻辑切成**纯**（好测）和**有副作用**（难测）两半，分别对待：

纯逻辑——单测直接覆盖：
```rust
r_candidate_paths_from("/x/bin:/y/bin")  // 含 /x/bin/R、/y/bin/R，还含已知位置 /usr/local/bin/R
r_candidate_paths_from("/usr/local/bin") // /usr/local/bin/R 只出现一次（PATH 与已知位置去重）
r_candidate_paths_from("/a::/b")         // 跳过空段，不产生裸 "R"
dedup_installs([4.5.2@a, 4.5.2@b, 4.4.0@c]) // 同版本去重，得 [4.5.2@a, 4.4.0@c]
```

有副作用（起进程）——用 `#[ignore]` 的真·测试覆盖，手动 / CI 之外跑：
```rust
#[ignore] fn discovers_at_least_one_r() { assert!(!discover().is_empty()) }
```

## 3. 实现到通过（TDD 绿）
- `RInstall { path, version }`：一处安装。
- `r_candidate_paths_from(path)`：`PATH` 里每个目录拼 `/R` + 一组 `KNOWN_R_PATHS`（`/usr/local/bin/R`、`/opt/homebrew/bin/R`、CRAN framework、Linux 路径），再 `dedup_keep_order` 去重保序。
- `probe_r(path)`：`Command::new(path).arg("--version").output()`，把输出喂给 Step 29 的 `parse_r_version`（stdout 不行就试 stderr）。
- `discover()`：候选路径 `filter_map(probe_r)` 再 `dedup_installs` 按版本去重。

本机实测：`cargo test --lib -- --ignored discovers_at_least_one_r` → **1 passed**（发现了 `/usr/local/bin/R` = 4.5.2）。纯逻辑 8 个单测全绿。

## 4. 改了哪些文件 / 加了什么
- `src/rversion.rs`：新增 `RInstall`、`KNOWN_R_PATHS`、`dedup_keep_order`、`r_candidate_paths_from`、`dedup_installs`、`probe_r`、`discover`，以及 5 个新测试（含 1 个 `#[ignore]`）。

## 5. 学到的语法 / 技巧
- **`Command::new(path).arg("--version").output()`**：起一个子进程、**等它结束**、一次性收集 stdout/stderr/退出码（`output()` 等于"跑完拿全部"；之前 `install.rs` 用的 `status()` 是"跑完只拿退出码"）。
- **`.ok()?`**：`Result` → `Option` 再 `?`——子进程起不来（路径不存在）就让 `probe_r` 返回 `None`，不 panic。
- **`filter_map(probe_r)`**：对每个候选路径探测，自动**丢掉** `None`、**留下并解包** `Some` 里的值。一步完成"过滤 + 变换"。
- **`HashSet::insert` 当去重器**：`insert` 返回 `bool`（第一次见 → `true`）。`filter(|x| seen.insert(x.clone()))` 是"保序去重"的惯用法——比排序去重更适合"想保留优先级顺序"的场景。
- **`String::from_utf8_lossy(&bytes)`**：进程输出是原始字节（`Vec<u8>`），不保证是合法 UTF-8；`from_utf8_lossy` 把坏字节替换成 `�` 而非报错，稳。

## 6. 设计巧思 / 方法论
- **纯 / 不纯分治**：把"算候选路径""按版本去重"做成纯函数，单测能脱离真实环境覆盖大部分逻辑；只把"起进程"这一点点副作用留给 `#[ignore]` 测试。**测试金字塔**：大量快而纯的单测在底，少量慢而真的集成测试在顶。
- **优先级即顺序**：候选路径的顺序就是优先级（`PATH` 在前、已知位置在后）；`dedup_installs` 保留"先出现的"，于是"同一版本有多份安装"时，留下的是 `PATH` 上更优先的那个。**用数据结构的顺序编码策略**，比写一堆 if 更干净。

## 7. 领域知识（R / 工具链）
- **R 装在哪**：macOS 上 Homebrew 装到 `/usr/local/bin`（Intel）或 `/opt/homebrew/bin`（Apple Silicon）；CRAN 官方 `.pkg` 装到 `/Library/Frameworks/R.framework/...`。Linux 多在 `/usr/bin` 或 `/usr/lib/R/bin`。本机是 Homebrew 的 `/usr/local/bin/R`。
- **同一版本多份**：一台机器上完全可能既有 Homebrew R 又有 CRAN R，甚至同版本两份；所以"按版本去重"是必要的。
- **rig**：R 社区的版本管理器（类似 pyenv）。它切换 framework 软链来换默认版本；更完整的发现可以去 glob `R.framework/Versions/*`，留作后续增强。

## 8. 软件设计理念
- **发现要稳健、不要假设**：探测失败（路径不存在、不是 R、输出认不出）一律安静地跳过，而不是崩。面对真实多样的机器环境，"尽力而为 + 优雅降级"比"假设理想环境"靠谱得多——这和 Step I 缓存"读不到就当没有"是同一种态度。

## 9. 小测验（自测）
1. 为什么把 `r_candidate_paths_from` 设计成"接收 `PATH` 字符串参数"，而不是直接在函数里读环境变量？
2. `filter(|x| seen.insert(x.clone()))` 为什么能去重？`insert` 的返回值是什么含义？
3. `probe_r` 里为什么 stdout 解析失败还要再试 stderr？
4. 一台机器上 `/usr/local/bin/R` 和 CRAN framework 里的 R 是同一个 4.5.2 版本，`discover()` 会返回几条？为什么？返回的是哪一条？

## 10. 参考答案
1. 传参让它变成**纯函数**：给定输入必得相同输出，不依赖运行环境，单测可随意构造 `PATH` 字符串。读真实环境变量的活由 `discover()` 这一层做（把副作用推到边缘）。
2. `HashSet::insert(x)` 在 `x` **第一次**出现时返回 `true`、之后返回 `false`；`filter` 只保留返回 `true` 的元素，于是每个值只过一次——且保持原顺序。
3. `R --version` 绝大多数构建打到 stdout，但不保证；两处都试是**稳健性**兜底，避免因个别环境差异而"发现不到"。
4. 返回**一条**。`discover()` 末尾按版本 `dedup_installs` 去重，同一 4.5.2 只留候选顺序里**先出现**的那条（`PATH` 优先于已知位置，故通常是 `/usr/local/bin/R`）。

## 11. 下一步预告
Step 31：项目级**钉版本**——读写 `.R-version` 文件（对标 `.python-version`），让一个项目记住"我要用哪个 R"。
