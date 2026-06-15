# Step 13：传递依赖（递归解析整棵依赖树）

> 模块：D 依赖求解 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-13-transitive-resolution.md`）｜ 测试：✅ 17 passed ｜ 上一步：[Step 12](step-12-best-match.md)

## 0. 一句话目标
从一个根包出发，沿 `Depends`/`Imports` 把**所有传递依赖**都解出来，得到"包名 → 选定版本"的集合。

## 1. 前置回顾
Step 12 的 `best_match` 能为单个依赖选一个版本。本步把它**递归地**应用到整棵依赖树：解根包 → 解它的依赖 → 再解依赖的依赖……直到全部确定。

## 2. 先写测试（TDD·红）
```rust
#[test]
fn resolves_transitive_deps() {
    let idx = index(); // pkgB Depends: R (>= 3.0.0), pkgA (>= 1.1.0); pkgA 有 1.0.0/1.2.0
    let res = resolve(&idx, "pkgB").unwrap();
    assert_eq!(res["pkgB"], Version::parse("2.0.0").unwrap());
    assert_eq!(res["pkgA"], Version::parse("1.2.0").unwrap()); // 满足 >=1.1.0 的最高
    assert!(!res.contains_key("R"));                            // R 伪依赖被跳过
}
```
（实现 `resolve` 前编译失败；红从略。）

## 3. 实现到通过（TDD·绿）
用一个**工作队列**迭代地展开依赖（比递归函数更不易踩借用/栈深问题）：
```rust
pub fn resolve(index: &PackageIndex, root: &str) -> Option<BTreeMap<String, Version>> {
    let mut resolved: BTreeMap<String, Version> = BTreeMap::new();
    let mut queue: Vec<(String, Option<Constraint>)> = vec![(root.to_string(), None)];
    while let Some((name, constraint)) = queue.pop() {
        if name == "R" || resolved.contains_key(&name) {
            continue;                                  // 跳过 R、跳过已解析
        }
        let pkg = best_match(index, &name, constraint.as_ref())?; // 找不到/不满足 → None
        resolved.insert(name, pkg.version.clone());
        for dep in &pkg.depends {
            queue.push((dep.name.clone(), dep.constraint.clone()));
        }
    }
    Some(resolved)
}
```
为把版本/约束放进队列与结果集，给 `Version`、`Constraint` 加了 `#[derive(Clone)]`。
`cargo test` → ✅ 17 passed（含"缺失依赖返回 None"的用例）。

## 4. 改了哪些文件 / 加了什么
- `src/resolver.rs`：新增 `resolve` + 2 个测试（递归解析、缺包返回 None）。
- `src/version.rs`：`Version`、`Constraint` 增加 `Clone` 派生（求解结果需要拥有版本/约束）。

## 5. 学到的语法 / 技巧
- **`BTreeMap<String, Version>`**：用有序表存"包名 → 版本"，迭代顺序稳定。
- **`Vec` 当栈用**：`queue.push(...)` / `while let Some(x) = queue.pop()` —— 经典的工作队列写法。
- **`?` 提前返回**：`best_match(...)?` 一旦为 `None`，整个 `resolve` 立刻返回 `None`。
- **`.clone()` / `.as_ref()`**：把所有权数据放进集合要 `clone`；`Option<Constraint>` 借出用 `as_ref()`。
- **`#[derive(Clone)]`**：让类型可被克隆。

## 6. 语言设计巧思
- **显式工作队列 vs 递归**：用 `Vec` 当工作表，把"递归"改写成"循环 + 队列"。好处：不受调用栈深度限制，也避免递归里同时持有多个借用的麻烦——Rust 的借用检查器会"逼"你写出更清晰的数据流。
- **`Clone` 的代价是显式的**：Rust 不会偷偷复制；要复制就显式 `.clone()`。这让性能开销**可见**，你能一眼看出哪里发生了拷贝。

## 7. 领域知识
- **传递依赖闭包**：装一个包，要把它依赖的、以及依赖的依赖……全部确定下来。
- **R 伪依赖**：`Depends: R (>= x)` 表示"需要某版本的 R 解释器"，不是一个要安装的包，故跳过（R 版本管理是另一回事）。
- **本步是简化版**：一旦某包被解析就不再重选；若两处对同一包提出**冲突**约束，目前不处理——这正是 Step 14 要补的。真实求解器（pubgrub）会回溯、给出冲突解释。

## 8. 软件设计理念
- **先简单可用，再迭代求精**：先把"无冲突的 happy path"跑通（Step 13），再加冲突检测（Step 14）。小步前进、每步可测。
- **worklist 算法**：用"待办队列 + 已完成集合"展开图遍历，是处理依赖图、任务图的通用套路。

## 9. 小测验（自测）
1. 为什么用"队列 + 循环"而不是递归函数来展开依赖？
2. `resolved.contains_key(&name)` 的作用是什么？
3. 为什么要给 `Version`/`Constraint` 加 `Clone`？
4. 本步在哪个情形下会返回 `None`？哪个情形被它**故意忽略**了（留给下一步）？

## 10. 参考答案
1. 队列不受调用栈深度限制，且避免递归中同时持有多个借用导致的借用冲突；数据流也更直观。
2. 去重：一个包已解析过就不再重复解析，避免重复工作与潜在死循环。
3. 因为要把版本放进 `resolved`、把约束放进 `queue`，这些集合需要**拥有**数据，而不是借用，故需可克隆。
4. 当某依赖在索引里**找不到**或**无满足版本**时返回 `None`。被故意忽略的是**版本冲突**（同一包被要求两个不兼容版本）——留待 Step 14。

## 11. 下一步预告
Step 14：冲突检测。把返回类型从 `Option` 升级为 `Result<Resolution, ResolveError>`，用一个**错误枚举**清楚地报告"找不到包 / 版本冲突"，让失败带上原因。
