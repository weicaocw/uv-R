# Step 33：命令行 `uvr r list / which / pin`

> 模块：M R 版本管理 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-33-r-cli.md`）｜ 产物：`src/main.rs`（`r_command` / `r_list` / `r_which` / `r_pin`）+ `RSelectError` 的 `Display` ｜ 上一步：[Step 32](step-32-select-r.md)

## 0. 一句话目标
把前几步的 R 版本能力接到终端：`uvr r list`（看有哪些）、`uvr r which`（看会用哪个）、`uvr r pin [版本]`（钉一个）。

## 1. 前置回顾
Step 29–32 在库里造好了发现 / pin / 选择的纯逻辑与薄包装（`resolve_r` 等）。但用户摸不到库——得有命令行。本步是**集成步**：CLI 只做"解析参数 → 调库 → 打印结果 / 错误"，不含新算法。

## 2. "测试"：可运行的演示（show, don't tell）
CLI 的分发逻辑用集成 / 手动演示验证（纯算法早在前几步单测覆盖了）。本机实跑：
```
$ uvr r list
* 4.5.2 /usr/local/bin/R          # * = 当前会选中的
$ uvr r pin 4.5                   # 钉到 4.5（部分版本）
已钉定 / pinned R 4.5 → .R-version
$ uvr r which
4.5.2 /usr/local/bin/R            # 4.5 前缀匹配到 4.5.2 ✔
$ uvr r pin                       # 无参数：钉当前解析到的版本
已钉定 / pinned R 4.5.2 → .R-version
$ uvr r pin 3.6 && uvr r which    # 钉一个没装的
钉定的 R 版本 3.6 未安装 / pinned R version 3.6 is not installed   (exit=1)
```
另加一个纯单测：`RSelectError` 的 `Display` 输出含中英文。

## 3. 实现到通过（TDD 绿）
- `main()` 增加分支 `Some("r") => r_command(&args[2..])`。
- `r_command` 二级分发到 `list` / `which` / `pin`。
- `r_list`：`discover()` 列出，用 `resolve_r(".")` 标 `*`。
- `r_which`：`resolve_r(".")`，成功打印、失败打印 `Display` 错误并退出码 1。
- `r_pin`：有参数就写它，没参数就写"当前解析到的版本"（便利功能，仿 `uv python pin`）。
- 给 `RSelectError` 实现 `Display`，让 CLI 打印人话而非 `{:?}`。
19 个测试（18 + 1 ignored）全绿；演示如上。

## 4. 改了哪些文件 / 加了什么
- `src/main.rs`：`r_command` + `r_list` / `r_which` / `r_pin`，`usage()` 增一行，模块文档更新。
- `src/rversion.rs`：`impl Display for RSelectError`（中英双语）+ 1 个测试。

## 5. 学到的语法 / 技巧
- **二级子命令分发**：`match rest.first().map(String::as_str)`——和顶层 `main` 同样的模式套一层，就有了 `uvr r <sub>`。子命令树就是这样长出来的。
- **`impl Display`（回顾 Step 05 的 trait）**：给自己的枚举实现 `std::fmt::Display`，于是 `println!("{e}")`、`.to_string()` 都能用。这正是"为什么类型要实现 trait"的实战——`#[derive(Debug)]` 给 `{:?}`（给程序员看），手写 `Display` 给 `{}`（给用户看）。
- **`is_some_and(|s| ...)`**：`Option` 的便捷方法——"是 `Some` 且内部满足条件"。比 `selected.as_ref().map(...).unwrap_or(false)` 干净。
- **`ExitCode` 表达成败**：每个子命令返回 `ExitCode::SUCCESS` / `FAILURE`，让 `uvr r which` 能在 shell 里 `&&` 串联、被脚本判断——CLI 要做"shell 公民"。
- **"无参数=用默认"的便利**：`r_pin` 里 `match args.first()` 区分"给了版本"和"没给"，没给就钉当前解析值——小细节，大体验。

## 6. 设计巧思 / 方法论
- **CLI 是薄壳**：所有真逻辑在库（`uvr::rversion`），`main.rs` 只搬运。好处：逻辑可被单测、可被别的前端（将来的 GUI / API）复用；CLI 改版不动核心。这是**端口与适配器**（六边形架构）的朴素版。
- **错误也是产品**：花力气给 `RSelectError` 写双语 `Display`、给正确的退出码。用户遇到的错误信息质量，直接决定工具好不好用。

## 7. 领域知识（R / 工具链）
- **对标 `uv python` / `rig`**：`uvr r list` ≈ `rig list` / `uv python list`；`uvr r which` ≈ `rig default` 的查询 / `uv python find`；`uvr r pin` ≈ 写 `.python-version`。我们用一套小命令覆盖了"看 / 选 / 钉"的日常。
- **`*` 标记的含义**：列表里被标星的，是"在**当前目录**下、按 pin > 最高 规则会被选中"的那个 R——和你 `cd` 到哪个项目有关（因为 pin 是项目级的）。

## 8. 软件设计理念
- **可发现性**：`usage()` 里列出 `uvr r ...`，用户 `uvr`（无参数）就能看到这组命令存在。功能再好，藏起来等于没有。
- **最小惊讶**：`pin` 写当前目录的 `.R-version`、`which` 读当前目录——和 git / uv 的"就近、项目级"心智模型一致，用户不必学新规矩。

## 9. 小测验（自测）
1. 为什么 `r_which` 失败时既打印错误又返回 `ExitCode::FAILURE`，而不是只打印？
2. `#[derive(Debug)]` 给的 `{:?}` 和手写 `Display` 给的 `{}`，面向的读者有何不同？为什么 CLI 要后者？
3. `uvr r pin 4.5` 后 `uvr r which` 得到 4.5.2——这背后是哪一步（哪个函数）的功劳？
4. 如果库逻辑都在 `uvr::rversion`，把 `main.rs` 整个删了，单元测试还能过吗？这说明了什么？

## 10. 参考答案
1. 退出码是 CLI 给**脚本 / shell** 的信号：`uvr r which || echo 装个R` 这种用法要靠非零退出码判断失败。只打印文字，机器读不懂成败。
2. `Debug`（`{:?}`）给**程序员**调试看，可能是 `PinnedNotFound("3.6")` 这种结构；`Display`（`{}`）给**用户**看，是"钉定的 R 版本 3.6 未安装"这种人话。CLI 面向用户，故要 `Display`。
3. Step 32 的 `version_matches` 前缀匹配：pin `4.5` 命中已装的 `4.5.2`，`select_r` 在匹配集合里取最高 → 4.5.2。CLI 只是把 `resolve_r` 的结果打印出来。
4. 能过。`rversion` 的测试只依赖库函数，与 `main.rs` 无关。这说明**核心逻辑与界面解耦**——库可独立测试、独立复用，CLI 只是它的一个使用者。

## 11. 下一步预告
Step 34：让 `install` 真正**用上**选中的那个 R（`R CMD INSTALL` 走 `resolve_r` 选出的可执行文件），并为 `uvr r install <版本>` 做资源墙交接——有 `rig` 就给出委托命令，没有就清楚地告诉你怎么办。模块 M 收尾、开 PR。
