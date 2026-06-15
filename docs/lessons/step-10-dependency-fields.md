# Step 10：依赖字段解析（复用 `Constraint`）

> 模块：B 元数据 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-10-dependency-fields.md`）｜ 测试：✅ 11 passed ｜ 上一步：[Step 09](step-09-dcf-parser.md)

## 0. 一句话目标
把依赖字段 `R (>= 2.15.0), xtable, pbapply (>= 1.3-2)` 拆成"包名 + 可选版本约束"的列表，**复用**模块 A 的 `Constraint`。

## 1. 前置回顾
Step 09 能把 DCF 解析成记录（字段→值）。但记录里的 `Depends` / `Imports` 仍是一整串文本。本步把这串拆成结构化的依赖项——这是依赖图、依赖求解的直接输入。这也是 `metadata` 与 `version` 两个模块第一次协作。

## 2. 先写测试（TDD·红）
```rust
#[test]
fn parses_name_and_constraint() {
    let deps = parse_dependencies("R (>= 2.15.0), xtable, pbapply (>= 1.3-2)");
    assert_eq!(deps.len(), 3);
    assert_eq!(deps[0].name, "R");
    assert!(deps[0].constraint.is_some());
    assert!(deps[1].constraint.is_none()); // xtable 无版本约束
}
```
实现 `Dependency` / `parse_dependencies` 之前，测试引用未定义类型，`cargo test` 会编译失败（`error[E0433]`）。（红从略，直接给实现。）

## 3. 实现到通过（TDD·绿）
```rust
use crate::version::Constraint;

#[derive(Debug)]
pub struct Dependency {
    pub name: String,
    pub constraint: Option<Constraint>,
}

pub fn parse_dependencies(field: &str) -> Vec<Dependency> {
    field
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(parse_one_dependency)
        .collect()
}

fn parse_one_dependency(entry: &str) -> Dependency {
    match (entry.find('('), entry.rfind(')')) {
        (Some(open), Some(close)) if close > open => Dependency {
            name: entry[..open].trim().to_string(),
            constraint: Constraint::parse(entry[open + 1..close].trim()),
        },
        _ => Dependency { name: entry.trim().to_string(), constraint: None },
    }
}
```
`cargo test` → ✅ 11 passed。其中一个测试还验证了解析出的约束**真的能用**：`R (>= 3.4.0)` 解析后，`matches(4.5.2)` 为真、`matches(3.0.0)` 为假。

## 4. 改了哪些文件 / 加了什么
- `src/metadata.rs`：新增 `use crate::version::Constraint;`、`struct Dependency`、`parse_dependencies`、`parse_one_dependency`，以及 2 个测试。

## 5. 学到的语法 / 技巧
- **跨模块复用**：`use crate::version::Constraint;` 把模块 A 的约束类型拿来用。
- **迭代器链**：`field.split(',').map(str::trim).filter(...).map(...).collect()`——惰性、可组合的数据流水线；`str::trim` 作为函数指针直接传给 `map`。
- **元组上的 `match` + 卫语句（guard）**：`match (a.find('('), a.rfind(')')) { (Some(open), Some(close)) if close > open => ... }`——一次性匹配"两个位置都找到且顺序合理"。
- **`Option<Constraint>` 字段**：用 `Option` 表达"可能没有版本约束"。
- **`.as_ref().unwrap()`**：从 `&Option<T>` 借出里面的值（测试里用）。

## 6. 语言设计巧思
- **迭代器是零成本抽象**：链式 `map/filter/collect` 读起来像高层描述，编译后却和手写循环一样快——Rust 鼓励这种"既清晰又高效"的写法。
- **类型表达可选性**：`Option<Constraint>` 让"无约束"成为类型层面的合法状态，调用方必须显式处理，不会漏。

## 7. 领域知识
- **依赖字段语法**：R 的依赖写成 `name` 或 `name (>= x.y)`，逗号分隔。
- **依赖类型**：`Depends`、`Imports`、`LinkingTo`、`Suggests`、`Enhances` 含义不同（如 `Imports` 是运行时必需但不附加到搜索路径，`Suggests` 是可选）。本步先统一解析其"语法形状"，语义区分留待求解阶段。
- 这与 Python `DESCRIPTION`/`pyproject` 的依赖声明同构——"名字 + 版本约束"是跨生态的通用骨架。

## 8. 软件设计理念
- **DRY（不要重复自己）**：版本约束的解析与匹配只在 `version` 模块写一次，`metadata` 直接复用——这正是 Step 08 拆出库 / 模块的回报。
- **单向数据流**：文本 → DCF 记录 → 依赖项，一层层精炼，每层职责单一。

## 9. 小测验（自测）
1. `parse_dependencies` 用迭代器链做了哪几步？
2. `Option<Constraint>` 为什么比"用空字符串表示无约束"更好？
3. `match (find('('), rfind(')')) { (Some(o), Some(c)) if c > o => ... }` 里的 `if c > o` 起什么作用？
4. 本步如何体现了 Step 08 拆模块的好处？

## 10. 参考答案
1. 按逗号切分 → 去空白（`trim`）→ 过滤空项 → 逐项解析成 `Dependency` → 收集成 `Vec`。
2. `Option` 让"无约束"成为类型层面明确、必须处理的状态；空字符串是"看起来有值其实无效"的隐患，容易误用。
3. 它确保确实存在一对括号且右括号在左括号之后（顺序合理），避免把畸形输入误当作"带约束"。
4. 约束逻辑在 `version` 模块写一次，`metadata` 通过 `use crate::version::Constraint` 直接复用，无需重写——模块化带来的复用。

## 11. 下一步预告
Step 11：把以上拼起来——定义 `Package`（名字 + 版本 + 依赖），把整个 `PACKAGES` 解析成"包名 → 包"的索引，能按名字查依赖。这就得到了**依赖图**，是模块 D（依赖求解）的输入。模块 B 收官。
