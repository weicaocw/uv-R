# Step 31：项目级钉版本 `.R-version`

> 模块：M R 版本管理 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-31-pin-r-version.md`）｜ 产物：`src/rversion.rs`（`PIN_FILE` / `pin_path` / `render_pin` / `parse_pin` / `read_pin` / `write_pin`）｜ 上一步：[Step 30](step-30-discover-r.md)

## 0. 一句话目标
让一个项目能用文件记住"我要用哪个 R"——读写 `.R-version`（对标 uv 的 `.python-version`）。

## 1. 前置回顾
Step 30 能发现机器上**有哪些** R 了。但"这个项目该用哪个"是另一回事——它应该跟着**项目**走、能进版本库、团队共享。uv 的做法是在项目根放一个 `.python-version` 文本文件；我们照搬，叫 `.R-version`。本步只做"读 / 写这个文件"。

## 2. "测试"先行（TDD 红）
还是纯 / 不纯分治。纯解析 / 渲染：
```rust
parse_pin(&render_pin("4.5.2"))      == Some("4.5.2")   // 往返
parse_pin("# 注释\n\n  4.4.0  \n")   == Some("4.4.0")   // 跳注释/空行、trim
parse_pin("# 只有注释\n\n")          == None            // 没有有效行
```
落盘往返（真读写，但只碰 `target/`，无需网络 / R，故不 `#[ignore]`）：
```rust
write_pin(dir, "4.5.2"); read_pin(dir) == Some("4.5.2")
```

## 3. 实现到通过（TDD 绿）
```rust
pub const PIN_FILE: &str = ".R-version";
pub fn pin_path(dir: &Path) -> PathBuf { dir.join(PIN_FILE) }
pub fn render_pin(spec: &str) -> String { format!("{}\n", spec.trim()) }
pub fn parse_pin(text: &str) -> Option<String> {
    text.lines().map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
}
pub fn read_pin(dir: &Path) -> Option<String> {
    parse_pin(&std::fs::read_to_string(pin_path(dir)).ok()?)
}
pub fn write_pin(dir: &Path, spec: &str) -> std::io::Result<()> {
    std::fs::write(pin_path(dir), render_pin(spec))
}
```
13 个测试（含此前的）全绿。

## 4. 改了哪些文件 / 加了什么
- `src/rversion.rs`：新增钉版本读写的 6 个项 + 4 个测试。

## 5. 学到的语法 / 技巧
- **`const PIN_FILE: &str = ".R-version";`**：编译期常量，`&'static str`，全程序共享、零成本。把"魔法字符串"提成一个有名字的常量，避免散落各处拼错。
- **迭代器链做解析**：`lines().map(str::trim).find(...)` ——把"逐行 → 去空白 → 找第一条有效行"写成一条声明式管道，没有手写循环和可变状态。
- **`str::trim` 作为函数值传给 `map`**：`map(str::trim)` 把"方法"当"函数指针"用，等价于 `map(|s| s.trim())` 但更简洁。
- **`.ok()?` 串联两种缺失**：`read_to_string(...).ok()?` 把"文件读不出"（`Err`）和后面"没有有效行"（`None`）统一成"返回 `None`"——读不到 pin 就当没钉，调用方好处理。
- **`std::io::Result<()>`**：写文件可能失败（磁盘满 / 没权限），返回 `Result` 让调用方决定怎么处理，而不是吞掉。

## 6. 设计巧思 / 方法论
- **格式向前兼容**：`parse_pin` 容忍注释（`#`）和空行。今天 pin 只有一行版本号，明天即便有人在文件里写了注释也不会崩。**解析要宽容、生成要规整**（render 只写干净的一行）——这是健壮文件格式的通用原则（postel 法则）。
- **纯逻辑可独立测**：`parse_pin` / `render_pin` 不碰磁盘，往返测试一句话搞定；只有 `read_pin` / `write_pin` 这层薄包装碰文件，且只写 `target/`。

## 7. 领域知识（R / 工具链）
- **`.python-version` 惯例**：pyenv / uv 用项目根的这个文件钉解释器版本，进版本库、团队统一。R 社区里 `rig` 也支持类似的项目级版本概念。uvr 用 `.R-version` 把这个好习惯带给 R。
- **为什么钉版本重要**：R 的小版本之间，包的二进制 ABI 不保证兼容（4.4 装的包，4.5 不一定能直接用）。把项目锁定到一个 R 版本，能避免"在我机器上好好的"这类问题——和锁包版本是同一种确定性诉求。
- **pin 可以是部分版本**：我们把 pin 存成自由字符串（如 `4.5.2` 或更宽的 `4.5`），把"怎么匹配"留给下一步，更灵活。

## 8. 软件设计理念
- **声明优于命令**：用一个文件**声明**意图（"用 4.5.2"），而不是每次命令行**手动**指定。声明能进版本库、可复现、自文档。这正是 uv / Cargo / lockfile 一脉相承的理念：把意图固化成文件。

## 9. 小测验（自测）
1. `render_pin` 里为什么要 `spec.trim()` 再加 `\n`？
2. `parse_pin` 用 `find` 而不是 `next`/取第一行，好处是什么？
3. `read_pin` 返回 `Option<String>` 而 `write_pin` 返回 `io::Result<()>`，为什么一个用 `Option` 一个用 `Result`？
4. 为什么落盘往返测试不用 `#[ignore]`，而 Step 30 的 `discover` 测试要 `#[ignore]`？

## 10. 参考答案
1. 去掉用户可能输入的首尾空白，保证写进文件的是干净的一行；补 `\n` 让文件以换行结尾（多数工具友好、`git` 不报"no newline"）。
2. `find` 会**跳过**前面的空行 / 注释，返回第一条**有效**行；直接取第一行遇到注释开头就废了。
3. 读"没钉版本"是**正常情况**（没有错误），用 `Option`（`None` = 没钉）；写文件**可能真的失败**（权限 / 磁盘），是错误，用 `Result` 把失败原因带出来。
4. 落盘往返只碰本地 `target/`（已 gitignore），无需网络或 R，确定且快，可进 CI；`discover` 需要机器上真的装了 R，环境相关，故 `#[ignore]`、手动跑。

## 11. 下一步预告
Step 32：**选哪个 R**——把"pin 的版本规格"和"发现到的安装清单"合起来，按优先级（pin 优先、否则取最高版本）选出最终该用的那一个 R。
