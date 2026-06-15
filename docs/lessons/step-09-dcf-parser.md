# Step 09：DCF 解析器（`PACKAGES` / `DESCRIPTION`）

> 模块：B 元数据 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-09-dcf-parser.md`）｜ 测试：✅ 9 passed + clippy 绿 ｜ 上一步：[Step 08](step-08-lib-and-modules.md)

## 0. 一句话目标
新增 `metadata` 模块，解析 R 的 **DCF 格式**——把 `PACKAGES` / `DESCRIPTION` 文本变成"一条条记录（字段→值）"。

## 1. 前置回顾
Step 08 把项目拆成了库 + 模块。现在加第一个真正的领域模块 `metadata`：R 仓库用 DCF 格式描述包的元数据，我们要先把它读成结构化数据，才能在后续步里抽出依赖、做求解。

## 2. 测试 + 一次真实的"红"（来自 clippy）
测试（用一段含两条记录、且有"折行续行"的样本）：
```rust
#[test]
fn parses_each_record() {
    let recs = parse(SAMPLE);
    assert_eq!(recs.len(), 2);
    assert_eq!(recs[0]["Package"], "A3");
}
#[test]
fn folds_continuation_lines() {
    let recs = parse(SAMPLE);
    assert!(recs[1]["Imports"].contains("shiny (>= 1.0.5)"));
}
```
`cargo test` 一上来就 9 passed。**但我们的 CI 关卡里还有 clippy，它当场报红：**
```
error: this `if` statement can be collapsed
  --> src/metadata.rs:32
  = note: `-D clippy::collapsible-if`
```
我最初写的是两层嵌套 `if let`：
```rust
if let Some(key) = &last_key {
    if let Some(val) = current.get_mut(key) { ... }
}
```
clippy 认为它可以合并——这就是一次真实的"红"（来自 lint，而非测试）。

## 3. 实现到通过（绿）
按 clippy 的建议，用 **let-chain**（Rust 2024 的 `if let A && let B`）合并：
```rust
if let Some(key) = &last_key
    && let Some(val) = current.get_mut(key)
{
    val.push(' ');
    val.push_str(line.trim());
}
```
再跑：clippy 绿、`cargo test` → ✅ 9 passed。解析器核心：
```rust
pub type Record = BTreeMap<String, String>;

pub fn parse(input: &str) -> Vec<Record> {
    let mut records = Vec::new();
    let mut current = Record::new();
    let mut last_key: Option<String> = None;
    for line in input.lines() {
        if line.trim().is_empty() {                 // 空行 = 记录分隔
            if !current.is_empty() { records.push(std::mem::take(&mut current)); }
            last_key = None;
            continue;
        }
        if line.starts_with(' ') || line.starts_with('\t') {  // 折行续行
            if let Some(key) = &last_key
                && let Some(val) = current.get_mut(key)
            { val.push(' '); val.push_str(line.trim()); }
            continue;
        }
        if let Some(idx) = line.find(':') {          // 字段: 值
            let key = line[..idx].trim().to_string();
            let value = line[idx + 1..].trim().to_string();
            current.insert(key.clone(), value);
            last_key = Some(key);
        }
    }
    if !current.is_empty() { records.push(current); }
    records
}
```

## 4. 改了哪些文件 / 加了什么
- 新增 `src/metadata.rs`：`pub fn parse` + `Record` 类型别名 + 3 个测试。
- `src/lib.rs`：新增 `pub mod metadata;`。

## 5. 学到的语法 / 技巧
- **`BTreeMap<String, String>`**：有序键值表（迭代顺序稳定，测试好写）；`type Record = ...` 是**类型别名**。
- **`input.lines()`**：按行迭代字符串。
- **`line.starts_with(' ')` / `line.find(':')`**：判前缀 / 找分隔符位置。
- **`&line[..idx]` 切片**：取字符串的一段。
- **`current.get_mut(key)`**：拿到值的可变引用以便追加。
- **`std::mem::take(&mut current)`**：把 `current` 取走（留下一个空的默认值），避免克隆——所有权技巧。
- **let-chain `if let A && let B { ... }`**：Rust 2024 起可在一个 `if` 里串多个 `let` 模式，省去嵌套。

## 6. 语言设计巧思
- **let-chain（Rust 2024）**：把"层层嵌套的 `if let`"拍平成一行链式条件，可读性更好——这是上一课讲的 **edition** 带来的新语法之一，老 edition 还用不了。
- **`std::mem::take` 与所有权**：Rust 不让你随便"移动走"借用中的值；`take` 用"换成默认值"的方式安全地取走所有权，零拷贝地把记录推入结果。

## 7. 领域知识
- **DCF 格式**：R 的 `PACKAGES`（仓库索引）和每个包的 `DESCRIPTION` 都用它。结构 = 一条条记录、记录内是 `字段: 值`、长值可"缩进续行"。
- 关键字段：`Package`、`Version`、`Depends`、`Imports`、`Suggests`、`License`、`NeedsCompilation` 等。
- 类比：R 仓库的 `PACKAGES` ≈ Python PyPI 的 "simple index"——一次抓取就拿到全量包的元数据，是依赖求解的原料。

## 8. 软件设计理念
- **解析与语义分离**：本步只把文本变成"通用记录（字段→值）"，**不**理解 `Depends` 的含义——那留给 Step 10。分层让每块都小而可测。
- **宽容解析（be liberal in what you accept）**：对不规整的行宽容忽略，提升鲁棒性。
- **lint 即老师**：把 CI 的 clippy 关卡当成代码评审，它推动我们用更地道的写法（let-chain）。

## 9. 小测验（自测）
1. DCF 里"记录"之间靠什么分隔？"折行续行"是怎样的行？
2. `std::mem::take(&mut current)` 解决了什么问题？
3. let-chain `if let A && let B` 相比嵌套 `if let` 好在哪？哪个 edition 起可用？
4. 为什么本步只把文本变成"字段→值"，而不顺便解析 `Depends` 的依赖项？

## 10. 参考答案
1. 记录之间用**空行**分隔；折行续行是**以空白（空格/制表符）开头**的行，属于上一字段值的延续，要拼回去。
2. 它能把借用中的 `current` 安全"取走"（替换为空的默认值）再推入结果，避免克隆、也绕过"不能移动借用值"的限制。
3. let-chain 把多层嵌套拍平成一行，可读性更好、缩进更浅；自 **Rust 2024 edition** 起可用。
4. 为了**分层**：先得到通用的结构化记录（与语义无关），再在下一步专门解析依赖字段。每步只做一件事，更小、更易测、更易复用。

## 11. 下一步预告
Step 10：解析依赖字段——把 `Depends: R (>= 2.15.0), xtable, pbapply` 这样的字符串，拆成"包名 + 可选版本约束"的列表，复用模块 A 的 `Constraint`。两个模块第一次协作。
