# Step 05：修正比较——`Eq`/`Ord` 一致性契约（结尾零陷阱）

> 模块：A 版本模型 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-05-compare-fix-contract.md`）｜ 测试：✅ 4 passed ｜ 上一步：[Step 04](step-04-compare-derive.md)

## 0. 一句话目标
修正版本比较：让 `1.0` 与 `1.0.0` 相等（R 语义），并让 `==` 与 `<`（即 `PartialEq` 与 `Ord`）**互相一致**。

## 1. 前置回顾
Step 04 用 `#[derive]` 让 `Version` 能比较，`1.2.3 < 1.10.0` 答对了。但 derive 出来的是"**结构化逐字段比较**"——对 `Vec` 来说，长度不同就算不相等。这在版本语义上是错的：`1.0` 和 `1.0.0` 应当相等。

## 2. 先写测试（TDD·红）
```rust
#[test]
fn trailing_zeros_are_equal() {
    assert_eq!(
        Version::parse("1.0").unwrap(),
        Version::parse("1.0.0").unwrap()
    );
}
```
`cargo test` → **断言失败（红，这次是逻辑错误，不是编译错误）**：
```
thread '...trailing_zeros_are_equal' panicked at src/main.rs:53:9:
assertion `left == right` failed
  left: Version { parts: [1, 0] }
 right: Version { parts: [1, 0, 0] }
```
derive 认为 `[1,0]` 与 `[1,0,0]` 不等——长度不同。这正是陷阱。

## 3. 实现到通过（TDD·绿）
移除 derive 的比较，**亲手实现**"零填充逐段比较"，并让相等性走同一套逻辑：
```rust
use std::cmp::Ordering;

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let n = self.parts.len().max(other.parts.len());
        for i in 0..n {
            let a = self.parts.get(i).copied().unwrap_or(0); // 越界用 0 补齐
            let b = other.parts.get(i).copied().unwrap_or(0);
            match a.cmp(&b) {
                Ordering::Equal => continue,
                non_eq => return non_eq,
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal // 相等性 = cmp 为 Equal
    }
}

impl Eq for Version {}
```
`cargo test` → ✅ `4 passed`（`trailing_zeros_are_equal` 通过，其余仍通过）。

## 4. 改了哪些文件 / 加了什么
- `src/main.rs`：① `#[derive(...)]` 去掉 `PartialEq, Eq, PartialOrd, Ord`，只留 `Debug`；② 新增 `use std::cmp::Ordering;`；③ 手写 `Ord`（零填充逐段比较）、`PartialOrd`（转发给 `Ord`）、`PartialEq`（走 `cmp`）、`Eq`；④ 新增测试 `trailing_zeros_are_equal`。

## 5. 学到的语法 / 技巧
- **手动 `impl Trait for Type`**：不再 derive，而是自己写 trait 实现，把领域规则（零填充）编码进去。
- **`self.parts.get(i)`**：按下标安全取值，返回 `Option<&u64>`（越界则 `None`）；`.copied()` 把 `Option<&u64>` 变 `Option<u64>`；`.unwrap_or(0)` 越界时用 `0` 补齐——这就是"零填充"。
- **`match a.cmp(&b)`**：两个数比较得 `Ordering`（`Less`/`Equal`/`Greater`）；相等就看下一段，不等就立即返回该结果。
- **`Option::partial_cmp` 返回 `Some(self.cmp(...))`**：把全序的 `cmp` 提升成"偏序"接口。

## 6. 语言设计巧思
- **`Eq`/`Ord` 一致性契约**：Rust 标准库要求 `a == b` 当且仅当 `a.cmp(b) == Equal`。Step 04 我们 derive 的 `PartialEq`（结构化、`[1,0] != [1,0,0]`）与若我们手写的零填充 `Ord`（视为相等）会**互相矛盾**——一旦把 `Version` 放进 `BTreeMap`、`BTreeSet` 或排序，就会行为错乱（找不到键、排序乱掉）。所以这一步**让 `PartialEq` 也走 `cmp`**，二者永远一致。
- **Rust 信任你、但契约要自己守**：编译器不强制 `Eq` 与 `Ord` 语义一致（它无法判断"语义"），这部分正确性由你保证——这正是"派生省事但可能语义不符"时，要果断改为手写的场景。

## 7. 领域知识
**版本相等的语义**：在 R / 多数包管理里，`1.0`、`1.0.0`、`1.0.0.0` 表示同一版本（尾部零不影响）。比较与相等都必须遵守这一点，否则依赖求解会把"同一个版本"当成两个，导致误判。

## 8. 软件设计理念
- **契约式设计 / 维护不变量**：`Eq` 与 `Ord` 之间存在"必须一致"的不变量；我们用"让 `eq` 复用 `cmp`"这一手法，从结构上**保证**不变量永不被破坏（而不是靠小心翼翼地手动同步两套逻辑）。
- **先甜后苦（迭代求精）**：先用 derive 快速跑通（Step 04），再用一个边界用例暴露其不足、改为正确实现（Step 05）。这是真实工程的常态：先简单可用，再被反例驱动着改对。

## 9. 小测验（自测）
1. derive 出来的比较为什么认为 `1.0 ≠ 1.0.0`？
2. "零填充"在代码里具体是哪一句实现的？
3. 为什么我们不直接 derive `PartialEq`、只手写 `Ord`？会出什么问题？
4. `PartialEq::eq` 为什么写成 `self.cmp(other) == Ordering::Equal`，而不是再比一遍字段？

## 10. 参考答案
1. derive 是"结构化逐字段比较"，对 `Vec` 来说长度不同即不等；`[1,0]` 与 `[1,0,0]` 长度不同，故判不等。
2. `self.parts.get(i).copied().unwrap_or(0)`——下标越界时取 `0`，相当于把短的一方在尾部补零再比。
3. 会违反 Rust 的 `Eq`/`Ord` 一致性契约：derive 的 `==` 说不等、手写的 `cmp` 说相等，二者矛盾；把 `Version` 放进 `BTreeMap`/排序会出错。
4. 为了**保证 `==` 与 `cmp` 永远一致**：让相等性复用同一套比较逻辑，就不可能出现"两套逻辑各说各话"的不一致，是用结构消除 bug。

## 11. 下一步预告
Step 06：实现**版本约束**（如 `>= 1.2.0`、`<= 2.0`）。我们会用 `enum`（枚举）表示比较运算符、用 `match` 判断某版本是否满足约束——这是依赖求解的最后一块版本积木。
