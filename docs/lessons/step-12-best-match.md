# Step 12：选出满足约束的最高版本（首遇生命周期）

> 模块：D 依赖求解 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-12-best-match.md`）｜ 测试：✅ 15 passed ｜ 上一步：[Step 11](step-11-package-index.md)

## 0. 一句话目标
写 `best_match`：从依赖图（`PackageIndex`）里，挑出某个包**满足版本约束的最高版本**。

## 1. 前置回顾
模块 B 给了我们可查询的依赖图。求解的最小原子操作是："要 `pkgA`，约束 `>= 1.1.0`，给我选哪个版本？"——通常选**满足约束的最高版本**。本步实现它，并第一次正面接触 Rust 的**生命周期**。

## 2. 先写测试（TDD·红）
```rust
#[test]
fn picks_highest_when_unconstrained() {
    let idx = index();
    let p = best_match(&idx, "pkgA", None).unwrap();
    assert_eq!(p.version, Version::parse("1.2.0").unwrap());
}
```
（实现 `best_match` 前编译失败 `E0425`；红从略。）

## 3. 实现到通过（TDD·绿）
```rust
pub fn best_match<'a>(
    index: &'a PackageIndex,
    name: &str,
    constraint: Option<&Constraint>,
) -> Option<&'a Package> {
    index
        .versions_of(name)
        .iter()
        .filter(|pkg| constraint.is_none_or(|c| c.matches(&pkg.version)))
        .max_by(|a, b| a.version.cmp(&b.version))
}
```
`cargo test` → ✅ 15 passed。测试覆盖：不限版本取最高、带约束取最高满足者、缺包或无满足版本时返回 `None`。

## 4. 改了哪些文件 / 加了什么
- 新增 `src/resolver.rs`：`pub fn best_match` + 3 个测试。
- `src/lib.rs`：新增 `pub mod resolver;`。

## 5. 学到的语法 / 技巧
- **生命周期 `'a`**：`fn best_match<'a>(index: &'a PackageIndex, ...) -> Option<&'a Package>`。它告诉编译器"返回的 `&Package` 借用自 `index`，活得和 `index` 一样久"。
- **`Option::is_none_or`**：`constraint.is_none_or(|c| c.matches(...))` —— 约束为 `None`（不限）时算通过，否则要满足约束。简洁表达"无约束 or 满足约束"。
- **迭代器 `filter` + `max_by`**：先按约束筛，再用 `max_by(比较器)` 取最大版本。
- **闭包 `|pkg| ...`、`|a, b| ...`**：内联的小函数。

## 6. 语言设计巧思
- **生命周期 = 借用安全的静态证明**：Rust 在编译期用生命周期保证"返回的引用不会比它指向的数据活得更久"，从而**根除悬垂指针**——别的语言要靠运行时 GC 或靠程序员自律，Rust 在编译期就证明了。
- **返回借用而非克隆**：`best_match` 返回 `&Package`，零拷贝；调用方只读。生命周期让这种"返回内部引用"既安全又高效。

## 7. 领域知识
- **版本选择策略**：依赖求解里，给定一个约束，常见策略是选"满足约束的最高版本"（也有生态偏好最低版本或锁定版本）。`best_match` 实现的就是"最高满足版本"。
- 这是求解的**原子操作**；把它对每个依赖递归应用，就能展开整棵依赖树（下一步）。

## 8. 软件设计理念
- **纯函数、易测**：`best_match` 无副作用——输入索引与约束，输出一个选择。纯函数最好测、最好复用、最好推理。
- **自底向上**：先做稳"选一个版本"，再在它之上搭"递归解整棵依赖树"。

## 9. 小测验（自测）
1. 签名里的 `'a` 是什么？它保证了什么？
2. `constraint.is_none_or(...)` 在约束为 `None` 时返回什么？为什么这正合需求？
3. `best_match` 为什么返回 `Option<&Package>` 而不是 `Option<Package>`？
4. `max_by` 在这里起什么作用？

## 10. 参考答案
1. `'a` 是生命周期参数；它把返回的 `&Package` 与入参 `index` 绑定，保证返回引用不会比 `index` 活得久（编译期杜绝悬垂引用）。
2. 返回 `true`（视为通过）。因为"无约束"应当接受任意版本，正是我们想要的语义。
3. 返回借用避免复制整个 `Package`；调用方只需读取，借用更高效，且生命周期保证安全。
4. 在所有满足约束的候选里，按版本取**最大**（最高版本）。

## 11. 下一步预告
Step 13：递归求解——从一个根包出发，沿 `Depends`/`Imports` 把**所有传递依赖**都解析出来，汇成"包名 → 选定版本"的集合；跳过对 R 本身的伪依赖，并识别"找不到的包"。
