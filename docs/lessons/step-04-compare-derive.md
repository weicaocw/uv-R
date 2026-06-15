# Step 04：让 `Version` 能比较大小（derive）

> 模块：A 版本模型 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-04-compare-derive.md`）｜ 测试：✅ 3 passed ｜ 上一步：[Step 03](step-03-parse-version.md)

## 0. 一句话目标
让两个 `Version` 能用 `<`、`>` 比较，且按**数字逐段**比（`1.2.3 < 1.10.0`），而不是按字符串比。

## 1. 前置回顾
Step 03 能把字符串解析成 `Version` 了。但"解依赖"（如 `>= 1.2.0`）的前提是能**比较版本**。本步补上"可比较"，并第一次正式认识 **trait**。

## 2. 先写测试（TDD·红）
```rust
#[test]
fn orders_numerically_not_lexically() {
    let a = Version::parse("1.2.3").unwrap();
    let b = Version::parse("1.10.0").unwrap();
    assert!(a < b, "1.2.3 应当小于 1.10.0");
}
```
此刻 `Version` 只有 `#[derive(Debug)]`，没有比较能力。`cargo test` → **编译失败（红）**：
```
error[E0369]: binary operation `<` cannot be applied to type `Version`
```

## 3. 实现到通过（TDD·绿）
只改一行——给 `Version` 派生四个比较 trait：
```rust
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Version { /* ... */ }
```
`cargo test` → ✅：
```
running 3 tests
test tests::orders_numerically_not_lexically ... ok
test tests::parses_dotted_version ... ok
test tests::rejects_non_numeric ... ok
test result: ok. 3 passed; 0 failed
```
**为什么对**：derive 生成"逐字段比较"，我们只有 `parts: Vec<u64>`，于是它**逐个整数比**：第 2 段 `2 < 10` → `1.2.3 < 1.10.0`。若按字符串比，`'2' > '1'` 会得出相反的错误结论——这正是造 `Version` 类型的意义。

## 4. 改了哪些文件 / 加了什么
- `src/main.rs`：① `#[derive(...)]` 增加 `PartialEq, Eq, PartialOrd, Ord`；② 新增测试 `orders_numerically_not_lexically`。
- 顺带：上一步那条 `parts is never read` 警告**消失了**——因为比较时真正读取了 `parts`。

## 5. 学到的语法 / 技巧
- **trait（特质）**：一个类型可"具备"的一组能力 / 接口。已见过 `Debug`（`{:?}` 打印）、`Display`（`{}` 打印）；比较能力由 `PartialEq`/`Eq`（`==`）、`PartialOrd`/`Ord`（`<`、`>`）提供。
- **`#[derive(...)]` 派生比较**：和派生 `Debug` 一样，编译器能自动生成"逐字段比较"的实现。
- **运算符受 trait 约束**：没有 `PartialOrd`，`<` 就用不了（这就是 §2 的 `E0369`）。

## 6. 语言设计巧思
Rust 把"能不能比较"做成**显式能力（trait）**，而非默认所有类型都能比。好处：编译器保证你只对"声明了可比较"的类型用 `<`，否则直接编译报错。这是 Rust"让错误在编译期暴露"的一贯风格。`Partial` 与非 `Partial`（`PartialOrd` vs `Ord`）牵涉"是否存在无法比较的值"（典型如浮点 `NaN`）——整数版本号没有这种情况，所以四个都能要。**下一步我们会撞上和这套 trait 的"一致性契约"相关的真实陷阱。**

## 7. 领域知识
**版本号语义**：包管理里版本几乎都"按段、按数字"比较，否则 `1.10` 会被错判小于 `1.2`。这直接影响依赖求解（"满足 `>= 1.2.0` 的最高版本"）。不同生态规则略异（R、SemVer……），但"逐段数字比较"是共同内核。

## 8. 软件设计理念
- **面向能力 / 逻辑内聚到类型**：不在调用处堆 `if` 比版本，而是把"可比较"这一能力赋予类型本身，调用处直接 `a < b`。逻辑内聚、自动复用——"单一职责"与"面向接口（trait）"的体现。

## 9. 小测验（自测）
1. 为什么 `"1.2.3" < "1.10.0"`（字符串比较）得到"假"，而我们想要"真"？
2. `#[derive(PartialOrd)]` 对只有 `parts: Vec<u64>` 的结构体，是怎么比较两个值的？
3. 只留 `#[derive(Debug)]`、删掉比较 trait，§2 的测试会怎样？报什么错？
4. 为什么上一步的 `parts is never read` 警告这一步消失了？

## 10. 参考答案
1. 字符串逐字符比，第三个字符 `'2'`（0x32）> `'1'`（0x31），故判 `"1.2.3" > "1.10.0"`；但版本要按数字比第二段 `2 < 10`，应为真。
2. 逐字段比较；只有 `parts`，于是用 `Vec` 的比较：从头逐元素比，第一个不等的元素决定大小（`[1,2,3]` vs `[1,10,0]` 在第二元素 `2<10` 分胜负）。
3. 编译失败，报 `error[E0369]: binary operation < cannot be applied to type Version`，因为没有 `PartialOrd` 就没有 `<` 能力。
4. 因为 `a < b` 的比较真正**读取**了 `parts`（之前只写不读，才被警告）。

## 11. 下一步预告
Step 05：写一个针对**结尾零**的测试（`1.0` 应当等于 `1.0.0`），会发现这套 `derive` 比较在这种边界上**给错答案**；再亲手实现正确比较，并搞懂 `Eq`/`Ord` 之间"必须一致"的契约。（"先甜后苦"拆分模式的典型一课。）
