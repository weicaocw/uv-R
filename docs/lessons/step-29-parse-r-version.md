# Step 29：解析 `R --version` 输出（R 版本管理的第一块砖）

> 模块：M R 版本管理 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-29-parse-r-version.md`）｜ 产物：`src/rversion.rs` ｜ 上一步：[Step 28](step-28-end-to-end-benchmark.md)

## 0. 一句话目标
写一个**纯函数** `parse_r_version`：把 `R --version` 的输出（一段文本）解析成我们已有的 `Version`。

## 1. 前置回顾
到 v0.6 为止，uvr 只管"**包**"——它用的是 `PATH` 上的 `R`，从不关心那是哪个版本。但 uv 的招牌能力之一是管理**解释器版本**（`uv python install/pin`）。要让 uvr 也能"对标 uv 管 R 版本"，第一步得先**认得**一个 R 是什么版本。怎么问 R 它几岁？运行 `R --version`，它会打印：

```
R version 4.5.2 (2025-10-31) -- "[Not] Part in a Rumble"
```

本步只做一件事：从这段文本里**抠出** `4.5.2`。

## 2. "测试"：先用例子钉死行为（TDD 红）
我们先写测试，描述"想要什么"，再写实现。四条用例：

```rust
parse_r_version(r#"R version 4.5.2 (2025-10-31) -- "...""#) == Version::parse("4.5.2")
parse_r_version("R version 3.6.1")                          == Version::parse("3.6.1")
parse_r_version("R Under development (unstable)")           == None
parse_r_version("totally unrelated text")                  == None
```

最关键的一条（`takes_version_not_the_date`）：括号里的日期 `2025-10-31` 也含数字和 `-`，如果实现写歪了、抠成了 `2025.10.31` 就坏了。测试用 `assert_ne!` 把这个陷阱也钉死。

## 3. 实现到通过（TDD 绿）
```rust
pub fn parse_r_version(output: &str) -> Option<Version> {
    let mut tokens = output.split_whitespace();
    while let Some(tok) = tokens.next() {
        if tok == "version" {
            let candidate = tokens.next()?.trim_end_matches([',', ')']);
            return Version::parse(candidate);
        }
    }
    None
}
```

思路：把整段文本按空白切成词，**找到 `version` 这个词，取它后面紧跟的那个词**。`R version 4.5.2 (...)` 切出来是 `["R","version","4.5.2","(2025-10-31)",...]`——`version` 后面正是 `4.5.2`，而不是日期。跑测试：4 条全绿。

## 4. 改了哪些文件 / 加了什么
- 新增 `src/rversion.rs`：`parse_r_version` + 4 个单元测试。
- `src/lib.rs`：加一行 `pub mod rversion;`，把新模块挂到库里。

## 5. 学到的语法 / 技巧
- **迭代器 `split_whitespace()`**：按任意空白（空格 / Tab / 换行）切片，比 `split(' ')` 更稳——不会因为两个空格切出空字符串。
- **`while let Some(tok) = tokens.next()`**：手动驱动迭代器；循环体里**还能再调一次** `tokens.next()` 拿"下一个"词。这正是"取某词的后继"的惯用法。
- **`?` 作用在 `Option` 上**：`tokens.next()?` 如果是 `None`（`version` 是最后一个词、后面没东西了）就直接让整个函数返回 `None`。`?` 不只用于 `Result`，对 `Option` 同样好使。
- **`trim_end_matches([',', ')'])`**：传一个**字符数组**当"要削掉的集合"，防御性地削掉末尾可能粘着的 `,` 或 `)`。
- **复用而非另造**：解析结果直接喂给既有的 `Version::parse`，于是版本比较 / 相等 / 排序全都免费继承——这就是 Step 01–07 打地基的回报。

## 6. 设计巧思 / 方法论
- **纯函数优先**：`parse_r_version` 不碰进程、不碰文件、不碰网络——只是 `文本 → Option<Version>`。纯函数最好测（给输入、断言输出，没有任何环境依赖）。"运行 `R --version` 拿到那段文本"这种**有副作用**的活，留到后面的步骤，并用真实的 R 去跑。这叫**把 IO 推到边缘、把逻辑留在核心**。
- **用反例钉边界**：`assert_ne!(v, 2025.10.31)` 不是多余——它把"别抠成日期"这个隐藏陷阱变成一条会失败的测试。好测试既描述"要什么"，也描述"**不要**什么"。

## 7. 领域知识（R / 工具链）
- **怎么问 R 的版本**：`R --version` 打印到 stdout，首行就是 `R version X.Y.Z (date) -- "nickname"`。这是跨平台稳定的约定。
- **R 的版本号形态**：主流是 `4.5.2` 这种三段式；开发版会是 `R Under development (unstable)`（没有干净版本号）——我们对它返回 `None`，诚实地表示"认不出"。
- **对标 uv**：uv 用 `uv python install 3.12` 管 Python 版本；uvr 接下来要做的 `uvr r ...` 就是 R 版的对应物。能"认出版本"是这一切的前提。

## 8. 软件设计理念
- **小而可组合**：一个只做"解析一行文本"的函数，将来会被"发现系统里所有 R"（Step 30）反复调用。先把最小的可靠积木造出来，再拼大的。
- **失败要显式**：返回 `Option<Version>` 而不是"猜一个默认值"。认不出就 `None`，让调用方决定怎么办——绝不假装认得。

## 9. 小测验（自测）
1. 为什么用 `split_whitespace()` 而不是 `split(' ')`？
2. `tokens.next()?` 里的 `?` 在什么情况下会让函数提前返回？返回什么？
3. 若把实现写成"取第一个含数字的词"，`R version 4.5.2 (2025-10-31)` 会出什么问题？
4. 为什么把"运行 `R --version`"这一步留到以后，而不是塞进 `parse_r_version`？

## 10. 参考答案
1. `split_whitespace()` 把连续空白当一个分隔符且自动忽略首尾空白，不会切出空串；`split(' ')` 遇到两个空格会切出 `""`，还得自己过滤。
2. 当 `tokens.next()` 返回 `None`（即 `version` 是最后一个词、后面没有下一个词）时，`?` 让整个函数立即返回 `None`。
3. 第一个含数字的词仍是 `4.5.2`，看似没事；但若输入里版本前还有别的数字（如某些带 build 号的横幅），就会抠错。"取 `version` 之后那个词"语义更准，也顺带避开了括号里的日期。
4. 纯函数好测、可复用、无环境依赖；把"跑进程拿输出"的副作用留在边缘，核心逻辑就能脱离真实 R 被单测覆盖（见 Step 30 把真·探测放在 `#[ignore]` 测试里）。

## 11. 下一步预告
Step 30：**发现系统里所有的 R**——枚举候选路径（`PATH` + 已知安装位置），对每个跑 `R --version` 并用本步的 `parse_r_version` 解析，得到一张"路径 → 版本"的清单。
