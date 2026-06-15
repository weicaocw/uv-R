# Step 03：把字符串解析成 `Version`（Option / Result / match）

> 模块：A 版本模型 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-03-parse-version.md`）｜ 测试：✅ 2 passed ｜ 上一步：[Step 02](step-02-version-struct.md)

## 0. 一句话目标
写一个函数，把字符串 `"1.2.3"` 解析成 `Version`；遇到不合法输入（如 `"1.2.x"`）返回"没有"。

## 1. 前置回顾
Step 02 我们有了 `Version` 类型，但实例是手写 `vec![1,2,3]` 拼的。真实世界从仓库拿到的是字符串。本步补上"字符串 → Version"，并第一次走真正的 **TDD**。

## 2. 先写测试（TDD·红）
先写测试，描述我们要的行为——此时 `parse` 还不存在：
```rust
#[test]
fn parses_dotted_version() {
    let v = Version::parse("1.2.3").unwrap();
    assert_eq!(v.parts, vec![1, 2, 3]);
}
```
`cargo test` → **编译失败（红）**：
```
error[E0599]: no associated function ... named `parse` found for struct `Version`
```
红得明明白白：功能还不存在。

## 3. 实现到通过（TDD·绿）
补上 `parse`，再加一个测试覆盖"失败返回 None"：
```rust
impl Version {
    fn parse(s: &str) -> Option<Version> {
        let mut parts = Vec::new();
        for piece in s.split(['.', '-']) {
            match piece.parse::<u64>() {
                Ok(number) => parts.push(number),
                Err(_) => return None,
            }
        }
        Some(Version { parts })
    }
}
```
`cargo test` → ✅：
```
running 2 tests
test tests::parses_dotted_version ... ok
test tests::rejects_non_numeric ... ok
test result: ok. 2 passed; 0 failed
```
`cargo run` 的演示：`Some(Version { parts: [1, 2, 3] })` 与 `None`。

## 4. 改了哪些文件 / 加了什么
- `src/main.rs`：① 新增 `impl Version { fn parse(...) -> Option<Version> }`；② `main` 改为演示解析成功/失败；③ 新增两个测试 `parses_dotted_version`、`rejects_non_numeric`。

## 5. 学到的语法 / 技巧
- **`impl Version { ... }`**：给类型实现方法 / 关联函数的地方。
- **`fn parse(s: &str) -> Option<Version>`**：`s: &str` 接收"一段（借来的、只读的）文字"；返回 `Option<Version>`。
- **`Option<T>`**：`Some(值)` 或 `None`——表达"可能有、可能没有"，取代危险的 null。
- **`Result`（来自 `.parse::<u64>()`）**：`Ok(值)` 或 `Err(错误)`——表达"可能成功、可能失败带原因"。
- **`match`**：按不同情况分支处理；`Ok(number) => ...`、`Err(_) => ...`，`_` 表示"不关心、丢弃"。
- **`for piece in s.split(['.', '-'])`**：按 `.` 或 `-` 切段并逐段遍历（所以 `"3.4-1"` 也能正确切成 3、4、1）。
- **`Version { parts }`**：字段简写（变量名与字段名同名时省略 `parts: parts`）。
- **`Version::parse(...)`**：用 `类型::函数` 调用"关联函数"。
- **测试相关**：`#[cfg(test)] mod tests` 是只在测试时编译的模块；`#[test]` 标记测试；`assert_eq!(a, b)` 断言相等；`assert!(cond)` 断言为真；`.unwrap()` 取出 `Some` 的值（遇 `None` 会 panic）；`.is_none()` 判断是否为 `None`。

## 6. 语言设计巧思
- **没有 null、没有异常满天飞**：Rust 把"可能没有 / 可能出错"编码进**类型**（`Option` / `Result`），编译器强制你处理这些情况，于是这类 bug 在编译期就被堵住，而不是运行时炸。
- **`&str` 是"借用"**：`parse` 只是**借看**字符串、不拿走它的所有权，所以调用方调用后仍能用自己的字符串。这是 Rust 所有权 / 借用体系的一角（后续展开）。

## 7. 领域知识
**版本字符串的解析**：R / 包管理里版本用 `.` 或 `-` 分隔（如 `3.4-1`）。把它切段并逐段转成整数，是"逐段数字比较"和"约束匹配"的前置步骤。非法版本要能被识别（返回 `None`），以免污染后续求解。

## 8. 软件设计理念
- **让非法状态不可表达 / 显式错误**：用 `Option` 让"解析失败"成为类型层面必须处理的结果，而不是返回一个"看起来正常其实非法"的值。
- **happy-path 与失败路径分离**：`match` 把成功与失败两条路写得清清楚楚，可读性与正确性都更好。

## 9. 小测验（自测）
1. `Option` 和 `Result` 各表达什么？它们的取值分别叫什么？
2. `parse("1.2.x")` 为什么返回 `None`？是哪一行让它返回的？
3. `Err(_)` 里的 `_` 是什么意思？
4. `.unwrap()` 什么时候会让程序崩溃？测试里为什么用它却很安全？

## 10. 参考答案
1. `Option<T>`：可能有（`Some(值)`）或没有（`None`）。`Result<T,E>`：成功（`Ok(值)`）或失败（`Err(错误)`）。
2. 因为 `"x"` 不能转成数字，`"x".parse::<u64>()` 返回 `Err(...)`，命中 `Err(_) => return None`，整体解析放弃、返回 `None`。
3. `_` 是通配，表示"这里有个值但我不关心它的内容"——这里指不关心具体的解析错误。
4. 当它作用在 `None` 上时会 panic。测试里输入是写死的合法串，必然 `Some`，所以安全；处理真实可能失败的输入时不应随意用 `unwrap`。

## 11. 下一步预告
Step 04：让两个 `Version` 能比较大小（`1.2.3 < 1.10.0`）。先用最省事的 `#[derive]` 实现比较，并第一次正式认识 **trait**。（那条 `parts is never read` 警告也会在比较时消失。）
