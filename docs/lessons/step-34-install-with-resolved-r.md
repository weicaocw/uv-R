# Step 34：用选中的 R 安装 + `uvr r install` 资源墙交接

> 模块：M R 版本管理 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-34-install-with-resolved-r.md`）｜ 产物：`install.rs` / `commands.rs` / `main.rs` 串入 `r_bin`，`rversion.rs` 加 `rig_available` / `rig_install_hint` ｜ 上一步：[Step 33](step-33-r-cli.md)

## 0. 一句话目标
两件事收尾模块 M：①让 `uvr install` 真正**用上**解析出来的那个 R；②`uvr r install <版本>` 把"装新 R"这件系统级的活**诚实地交还**（委托 rig 或给指引）。

## 1. 前置回顾
Step 32 的 `resolve_r(dir)` 已能算出"该用哪个 R"，Step 33 让你能在终端 `pin`。但 `install` 仍写死 `Command::new("R")`——pin 了也没用上。本步把 `resolve_r` 的结果**贯通**到 `R CMD INSTALL`。

## 2. "测试"：可运行的演示 + 纯单测
纯单测（`rig_install_hint`，不碰系统）：
```rust
rig_install_hint("4.4", true).contains("rig add 4.4")   // 有 rig → 给委托命令
rig_install_hint("4.4", false).contains("rig")          // 没 rig → 仍指向 rig + CRAN
```
本机演示：
```
$ uvr r install 4.4            # 本机没装 rig
uvr 不自己安装 R（系统级操作）/ uvr does not install R itself.
推荐用 rig ... rig add 4.4 ... 或从 CRAN 下载       (exit=1)

$ uvr install --repo <r-universe> dotenv --lib /tmp/...
→ 使用 R / using R 4.5.2: /usr/local/bin/R          # ← 新增：用解析出的 R
... R CMD INSTALL 正常装好 ...
```
全量 52 个测试（3 ignored）全绿。

## 3. 实现到通过（TDD 绿）
- **贯通 `r_bin`**：给 `install::install_tarball(tarball, lib, r_bin)` 和 `commands::install_packages(..., r_bin)` 加一个参数；`Command::new("R")` → `Command::new(r_bin)`。
- **`main::install` 解析 R**：调 `resolve_r(".")`，成功就打印 `→ 使用 R ...` 并把 `r.path` 传下去；失败（如 pin 了没装）就**报错中止**，绝不偷偷换一个 R。
- **资源墙**：`rig_available()`（跑 `rig --version`）+ 纯 `rig_install_hint(spec, present)`；`uvr r install` 打印指引并返回失败码。

## 4. 改了哪些文件 / 加了什么
- `src/install.rs`：`install_tarball` 增 `r_bin: &Path`，用它起进程；更新 `#[ignore]` 测试调用。
- `src/commands.rs`：`install_packages` 增 `r_bin` 透传。
- `src/rversion.rs`：`rig_available` + `rig_install_hint` + 1 个测试。
- `src/main.rs`：`install` 先 `resolve_r` 再装并打印用的是哪个 R；`r_command` 增 `install` 分支与 `r_install`；usage 更新。

## 5. 学到的语法 / 技巧
- **参数化外部命令**：`Command::new(r_bin)` 而非写死 `"R"`——把"用哪个可执行文件"变成数据。同一段安装逻辑，喂不同的 `r_bin` 就用不同的 R，零分支。
- **`let Some(spec) = args.first() else { ... return ... }`**：let-else 语法——拿不到就走 `else` 提前返回，主流程不必再套一层缩进。
- **`.map(|o| o.status.success()).unwrap_or(false)`**：把"命令跑没跑成功"安全地归约成 `bool`：跑不起来（`Err`）也当 false。探测外部工具是否存在的稳健写法。
- **"中止而非降级"的取舍**：`resolve_r` 失败时 `install` 选择**报错退出**，而不是 fallback 到随便一个 R。有时候"正确地失败"比"凑合着继续"更重要——尤其当用户**显式 pin** 了版本。

## 6. 设计巧思 / 方法论
- **依赖注入（朴素版）**：把 `r_bin` 从最外层（`main` 解析）一路传到最内层（`install_tarball` 起进程）。内层不关心"R 从哪来 / 怎么选的"，只接收一个路径。这让内层纯粹、可测（测试传 `Path::new("R")` 即可），选择策略集中在外层。
- **诚实的边界**：`uvr r install` 不假装能装 R。装 R 要管理员权限、要下载几百 MB——这是**真实的资源墙**。把它**交给专门的工具（rig）或交还用户**，并返回失败码让脚本知道"没装成"。这与项目一以贯之的"绝不擅自污染 / 改动系统环境"是同一条底线。

## 7. 领域知识（R / 工具链）
- **为什么"用哪个 R"对装包至关重要**：`R CMD INSTALL` 会把包编译 / 安装成**与该 R 版本匹配**的形态。用 4.5 装的包，未必能被 4.4 加载。所以"装包用的 R"必须正是项目 pin 的那个——本步把这个保证落地了。
- **rig 是什么**：r-lib 出的 R 版本管理器，能在一台机器上装 / 切多个 R 版本（mac/Win/Linux）。它处理官方安装器、权限、软链——正是"获取一个新 R"该用的专业工具。uvr 负责"用好已装的 R"，获取则委托给它，各司其职。

## 8. 软件设计理念
- **关注点分离 + 知止**：uvr 管"发现 / 选择 / 用 R 装包"，不碰"下载安装 R 解释器"。一个工具把自己该做的做到位、把不该做的明确交出去，比"什么都想自己干"更可靠、更可信。uv 也并不自己编译 Python——同一种克制。

## 9. 小测验（自测）
1. 把 `r_bin` 一路当参数传下去（而不是在 `install_tarball` 里读全局配置），带来哪些好处？
2. `install` 在 `resolve_r` 失败时为什么选择中止，而不是 fallback 到 `"R"`？什么场景下这个选择最重要？
3. `uvr r install` 为什么返回失败码（而不是成功）？对脚本意味着什么？
4. 为什么说"uvr 不自己装 R"反而是一个好的设计，而不是功能缺失？

## 10. 参考答案
1. 内层函数变纯、可单测（传 `Path::new("R")` 即可，不依赖任何全局状态）；"选哪个 R"的策略集中在外层一处，易改易测；同一逻辑能服务不同的 R，无需分支。
2. 因为用户可能**显式 pin** 了某版本；若 pin 的 R 没装，悄悄换一个 R 装包会产出"和预期不一致"的结果，违背确定性。`PinnedNotFound` 时报错中止，逼用户去装对的 R 或改 pin——这在"团队共享 `.R-version`、要求可复现"时最关键。
3. 因为它**没有真的装成 R**——只是给了指引。返回失败码让 `uvr r install X && uvr install ...` 这类链式脚本不会误以为 R 已就绪而继续。
4. 装 R 是系统级、需权限、平台各异的重活，已有 rig 这样的专业工具把它做好。uvr 专注"用好已装的 R"，把获取交出去，既守住"不擅自动系统"的底线，又避免重复造一个易错的轮子——这是**克制**与**关注点分离**，不是缺失。

## 11. 下一步预告
模块 M（R 版本管理）完成：发现 / 钉版本 / 选择 / 用选中的 R 装包 / 获取交接，全部到位。接下来开 PR，并写一份**中英对照用户手册** `docs/MANUAL.md`（模块 N），然后发布 v0.7。
