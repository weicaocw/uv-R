# Step 06：版本约束（enum + match）

> 模块：A 版本模型 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-06-version-constraints.md`）｜ 测试：✅ 6 passed ｜ 上一步：[Step 05](step-05-compare-fix-contract.md)

## 0. 一句话目标
表示并判断"版本约束"，如 `>= 1.2.0`：给定一个版本，回答它是否满足该约束。

## 1. 前置回顾
Step 03–05 我们能解析版本、并正确比较大小。依赖求解的语言层最后一块积木，就是**约束**——"我要 `>= 1.2.0` 的包"。本步把约束建模出来并实现判断。

## 2. 先写测试（TDD·红）
```rust
#[test]
fn constraint_ge_matches() {
    let c = Constraint::parse(">= 1.2.0").unwrap();
    assert!(c.matches(&Version::parse("1.10.0").unwrap()));
    assert!(!c.matches(&Version::parse("1.1.9").unwrap()));
}
```
实现 `Constraint` 之前，测试引用了尚不存在的类型，`cargo test` 会**编译失败（红）**：`error[E0433]: failed to resolve: use of undeclared type Constraint`。（红阶段从略，直接给实现。）

## 3. 实现到通过（TDD·绿）
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op { Lt, Le, Eq, Ge, Gt }

struct Constraint { op: Op, version: Version }

impl Constraint {
    fn parse(s: &str) -> Option<Constraint> {
        let s = s.trim();
        let (op, rest) = if let Some(r) = s.strip_prefix(">=") { (Op::Ge, r) }
            else if let Some(r) = s.strip_prefix("<=") { (Op::Le, r) }
            else if let Some(r) = s.strip_prefix("==") { (Op::Eq, r) }
            else if let Some(r) = s.strip_prefix('>')  { (Op::Gt, r) }
            else if let Some(r) = s.strip_prefix('<')  { (Op::Lt, r) }
            else if let Some(r) = s.strip_prefix('=')  { (Op::Eq, r) }
            else { return None; };
        Some(Constraint { op, version: Version::parse(rest.trim())? })
    }

    fn matches(&self, v: &Version) -> bool {
        let ord = v.cmp(&self.version);
        match self.op {
            Op::Lt => ord == Ordering::Less,
            Op::Le => ord != Ordering::Greater,
            Op::Eq => ord == Ordering::Equal,
            Op::Ge => ord != Ordering::Less,
            Op::Gt => ord == Ordering::Greater,
        }
    }
}
```
`cargo test` → ✅ `6 passed`。`cargo run` 演示：`... 满足 >= 1.2.0 ? true`。

## 4. 改了哪些文件 / 加了什么
- `src/main.rs`：① 新增 `enum Op`（五种比较运算符）；② 新增 `struct Constraint { op, version }`；③ `impl Constraint`：`parse`（解析约束串）与 `matches`（判断是否满足）；④ `main` 改为演示约束判断；⑤ 新增两个测试 `constraint_ge_matches`、`constraint_lt_uses_zero_padding`。

## 5. 学到的语法 / 技巧
- **`enum`（枚举）**：表示"只能是有限取值之一"。`Op` 只能是 `Lt/Le/Eq/Ge/Gt` 五者之一。
- **`#[derive(Clone, Copy)]` 之于枚举**：让 `match self.op` 能按值"复制"出来匹配（否则会试图从 `&self` 里移动 `op` 而报错）。
- **`match self.op { ... }`**：对枚举做**穷尽匹配**——五个分支全列出；漏一个编译器会报错。
- **`if let Some(r) = s.strip_prefix(">=")`**：`strip_prefix` 若以该前缀开头，返回 `Some(剩余部分)`，否则 `None`；`if let` 在是 `Some` 时取出 `r`。注意**两字符运算符要先判**，否则 `">="` 会被 `'>'` 抢先匹配。
- **`?` 运算符**：`Version::parse(rest.trim())?` —— 若解析得 `None`，整个函数立即返回 `None`；否则取出里面的 `Version` 继续。是处理 `Option`/`Result` 的简洁写法。

## 6. 语言设计巧思
- **用 `enum` 让非法状态不可表达**：运算符只可能是那五种，不可能出现"第六种非法运算符"。配合 `match` 的**穷尽性检查**，将来若给 `Op` 加一种取值，所有 `match` 会编译报错、逼你补上处理——编译器帮你堵住遗漏。
- **`?` 让错误传播无样板**：相比层层 `match`，`?` 把"失败就早退"压成一个符号，可读性高且不易漏处理。

## 7. 领域知识
**版本约束与依赖求解**：包的依赖往往写成 `pkg (>= 1.2.0)`。求解器要回答"哪些可用版本满足这些约束"。`matches` 正是这块判断的原子操作。注意 `< 2.0` 对 `2.0.0` 返回 false——因为我们 Step 05 的零填充让 `2.0.0 == 2.0`，约束语义才正确。

## 8. 软件设计理念
- **封闭集合用枚举，开放集合用其它**：运算符是封闭有限集 → `enum` 最贴切。
- **小而正交的积木**：`Version`（值）+ `Ord`（比较）+ `Constraint`（约束）各司其职、相互组合，为上层"依赖求解"打好地基——自底向上、逐步求精。

## 9. 小测验（自测）
1. 为什么解析时要**先**判 `>=`、`<=`，再判 `>`、`<`？
2. `match self.op` 如果只写了 4 个分支会怎样？
3. `Version::parse(rest.trim())?` 里的 `?` 做了什么？
4. 为什么 `< 2.0` 不匹配 `2.0.0`？这依赖我们前面哪一步的成果？

## 10. 参考答案
1. 因为 `">="` 也以 `'>'` 开头；若先判 `'>'`，`">= 1.2.0"` 会被当成 `'>'` 加上多余的 `"= 1.2.0"`，解析错误。先判两字符运算符可避免被单字符抢匹配。
2. 编译失败：`match` 对枚举要求**穷尽**，漏掉任一取值编译器会报 "non-exhaustive patterns"，逼你补全。
3. 若 `Version::parse(...)` 返回 `None`，`?` 让 `Constraint::parse` 立即返回 `None`；否则取出 `Version` 继续往下用。
4. 因为 `2.0.0` 与 `2.0` 相等（不小于），故不满足 `< 2.0`。这依赖 Step 05 的"零填充相等"实现。

## 11. 下一步预告
模块 A 的版本积木齐了。Step 07：给项目接上 **CI（GitHub Actions）**——每次推送 / PR 自动跑 `cargo test`，并把 CI 配置当成一课逐行讲清。然后开 PR 合入 `main`，暂停等你 review。
