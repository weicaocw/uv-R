# Step 20：`uvr install` 命令（端到端安装）

> 模块：E 下载+安装 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-20-install-command.md`）｜ 测试：✅ 26 passed + 真·安装 dotenv ｜ 上一步：[Step 19](step-19-download-install.md)

## 0. 一句话目标
把解析 + 下载 + 安装串成命令：`uvr install --repo <仓库> [--lib <目录>] <包>...`，把依赖闭包装进项目本地库。

## 1. 前置回顾
Step 19 能装单个 tarball。本步把"求解出整组包 → 逐个下载安装"接成一个命令，并让用户能指定本地库目录。

## 2. 先写测试（TDD·红）
把"要装哪些、各自 URL"做成**纯函数** `install_plan`（不触网、不安装），便于单测：
```rust
#[test]
fn builds_install_plan_with_urls() {
    let plan = install_plan(PACKAGES, &["pkgB".to_string()], "https://repo.example").unwrap();
    let pkgb = plan.iter().find(|i| i.name == "pkgB").unwrap();
    assert_eq!(pkgb.url, "https://repo.example/src/contrib/pkgB_2.0.0.tar.gz");
}
```

## 3. 实现到通过（TDD·绿）
- `install_plan`（纯）：解析闭包 → `Vec<InstallItem { name, version, url }>`。
- `install_packages`（IO）：按计划逐个 `download` + `install_tarball`。
- `main` 加 `install` 子命令 + 手写 `--repo/--lib` 参数解析。

常规 `cargo test` → ✅ 26 passed。**真·端到端**（dotenv 是纯 R、零依赖的小包）：
```
$ uvr install --repo https://gaborcsardi.r-universe.dev dotenv --lib ./r-lib
...
* DONE (dotenv)
installed dotenv 1.0.3.9000
→ 已安装到项目本地库: ./r-lib
```
`r-lib/dotenv/DESCRIPTION` 出现；全局 `.libPaths()` 未受影响。

## 4. 改了哪些文件 / 加了什么
- `src/commands.rs`：新增 `InstallItem`、纯函数 `install_plan`、IO 函数 `install_packages` + 1 个计划测试。
- `src/main.rs`：新增 `install` 子命令，手写 `--repo`/`--lib` 参数解析。
- `.gitignore`：忽略项目本地库目录 `/r-lib`。

## 5. 学到的语法 / 技巧
- **手写参数解析**：`while i < rest.len() { match rest[i].as_str() { "--repo" if ... => {...; i += 2;} other => {...; i += 1;} } }`。
- **`let-else`**：`let Some(repo) = repo else { ...; return ...; };`——取不出值就提前返回，主流程不必再缩进。
- **数据结构 `InstallItem`**：用一个小结构体承载"名字/版本/URL"，比裸元组更自描述。
- **`into_iter().map(...).collect()`**：把 `BTreeMap` 转成 `Vec<InstallItem>`。

## 6. 语言设计巧思
- **计划与执行分离**：`install_plan`（纯、可测、可"干跑"查看将装什么）与 `install_packages`（真正 IO）分开。纯计划能在没有网络/R 的情况下被测试与审查。
- **`let-else` 让错误路径扁平**：早退保持主逻辑不深嵌，可读性更好。

## 7. 领域知识
- **`install` 子命令**：对应 `pip install` / `uv add`——解析依赖并把它们装上。我们的版本装进**项目本地库**，对应 uv/renv 的项目级隔离。
- **依赖闭包安装**：要装的不只是目标包，还有它的（非自带）传递依赖；`install_plan` 给出完整清单。

## 8. 软件设计理念
- **计划/执行分离（plan vs apply）**：先算出"要做什么"（可测、可预览），再"去做"。这与 Terraform `plan`/`apply`、包管理器的 dry-run 同理，安全且可审查。
- **一个核心、多个入口**：`lock` 与 `install` 共享求解核心，只是末端动作不同（写 lockfile vs 下载安装）。

## 9. 小测验（自测）
1. 为什么把 `install_plan` 单独做成纯函数？
2. `let-else` 解决了什么可读性问题？
3. `uvr install` 默认把包装到哪里？为什么这样设计？
4. `install_plan` 返回的清单为什么不止目标包一个？

## 10. 参考答案
1. 它不触网、不安装，能在无网络/无 R 时被单测与审查（"将装哪些、从哪下"一目了然）；把不确定的 IO 隔离在 `install_packages`。
2. 避免"取值失败"时层层缩进；`let-else` 在失败分支直接 `return`，主逻辑保持扁平、易读。
3. 默认装进项目本地目录 `r-lib`（可用 `--lib` 改）。这样多项目隔离、可复现、不污染全局 R 环境。
4. 因为要安装的是**依赖闭包**——目标包加上它所有（非 R 自带的）传递依赖。

## 11. 下一步预告
模块 E 收官：`uvr install` 能从真实仓库把包及其依赖装进项目本地库。接着开 PR 合入 `main`。
之后做 **模块 G：benchmark**——用自写的计时脚本，把 `uvr` 的解析/求解与 `pak` 做对照，诚实地报告各自快在哪、平在哪。
