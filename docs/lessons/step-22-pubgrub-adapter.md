# Step 22：接入工业级求解器 pubgrub（带回溯）

> 模块：D′ pubgrub ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-22-pubgrub-adapter.md`）｜ 测试：✅ 28 passed（含 pubgrub 一致性 + 回溯）｜ 上一步：[Step 21](step-21-benchmark.md)

## 0. 一句话目标
把成熟的 [`pubgrub`](https://github.com/pubgrub-rs/pubgrub) 求解器接进来：它**会回溯**，能解开我们手写贪心版会误判为冲突的情形。手写版保留作对照与教学。

## 1. 前置回顾
Step 14 的手写求解器贪心、不回溯——遇到"换个版本本可共存"的情况会误报冲突。本步接入工业级 pubgrub，让 uvr 拥有真正的回溯求解能力，并用测试**当场对比**两者。

## 2. 先写测试（TDD·红 → 绿）
```rust
#[test]
fn pubgrub_agrees_with_handwritten() {       // 简单图上两者结果一致
    let idx = index();
    assert_eq!(resolve(&idx, "pkgB").unwrap(), resolve_pubgrub(&idx, "pkgB").unwrap());
}

#[test]
fn pubgrub_backtracks_where_greedy_conflicts() {
    // root 依赖 B、A；A 有 1.0/2.0；B 要求 A < 2.0。
    let idx = PackageIndex::from_packages_file(BACKTRACK_SAMPLE);
    assert!(matches!(resolve(&idx, "root"), Err(ResolveError::Conflict(_)))); // 贪心误报冲突
    let sol = resolve_pubgrub(&idx, "root").unwrap();                          // pubgrub 回溯成功
    assert_eq!(sol["A"], Version::parse("1.0.0").unwrap());
}
```
`cargo test` → ✅ 28 passed。**贪心报冲突、pubgrub 回溯把 A 选成 1.0.0** —— 一眼看出工业求解器的价值。

## 3. 实现到通过（TDD·绿）
- `cargo add pubgrub`（0.4）。用它内置的 `OfflineDependencyProvider` 省去手写整个 `DependencyProvider` trait：把依赖图灌进去，再调 `pubgrub::resolve`。
```rust
let mut dp = OfflineDependencyProvider::<String, Ranges<Version>>::new();
for pkg in index.packages() {
    let deps = pkg.depends.iter()
        .filter(|d| !is_builtin(&d.name))
        .map(|d| (d.name.clone(), constraint_to_range(d.constraint.as_ref())))
        .collect::<Vec<_>>();
    dp.add_dependencies(pkg.name.clone(), pkg.version.clone(), deps);
}
let root_version = best_match(index, root, None).ok_or(...)?.version.clone();
match pubgrub::resolve(&dp, root.to_string(), root_version) {
    Ok(sol) => Ok(sol.into_iter().collect()),
    Err(e)  => Err(ResolveError::NoSolution(format!("{e:?}"))),
}
```
- 约束 → pubgrub 版本集合：`>=`→`Ranges::higher_than`、`>`→`strictly_higher_than`、`<=`→`lower_than`、`<`→`strictly_lower_than`、`==`→`singleton`、无约束→`full`。
- **`Version` 补一个 `Hash`**：pubgrub 内部用 HashMap 按版本存依赖，需要 `Version: Hash`；而我们的 `Eq` 视 `1.0 == 1.0.0`，所以哈希前**先去掉尾部的 0**，让相等的版本哈希一致（否则违反 Hash/Eq 契约）。

## 4. 改了哪些文件 / 加了什么
- `Cargo.toml` / `Cargo.lock`：新增依赖 `pubgrub = "0.4"`。
- `src/resolver.rs`：新增 `resolve_pubgrub`、`constraint_to_range`、`ResolveError::NoSolution` + 2 个对比测试。
- `src/version.rs`：`Op` 改 `pub` + 加 `op()`/`version()` 访问器；`Version` 实现一致的 `Hash`。
- `src/metadata.rs`：`PackageIndex` 加 `packages()` 遍历器。

## 5. 学到的语法 / 技巧
- **用外部库的泛型类型**：`OfflineDependencyProvider::<String, Ranges<Version>>` —— 包名用 `String`、版本集合用 `Ranges<Version>`。
- **满足外部库的 trait 约束**：pubgrub 要求版本 `Clone + Ord + Debug + Display + Hash`、包名 `Clone + Eq + Hash + Debug + Display`——我们的类型逐一补齐。
- **手写一致的 `Hash`**：`impl std::hash::Hash for Version`，去尾零后再哈希。
- **把自己的模型翻译给外部库**：`constraint_to_range` 把我们的 `Constraint` 映射成 pubgrub 的 `Ranges`——这是**适配器**。

## 6. 语言设计巧思
- **`Hash` 必须与 `Eq` 一致**：与 Step 05 的 `Eq`/`Ord` 契约同源——标准库要求"相等的值哈希相等"。我们 `Eq` 视 `1.0 == 1.0.0`，故 `Hash` 也要让它们一致（去尾零）。**自定义相等语义时，`Eq`/`Ord`/`Hash` 必须步调一致**，否则 `HashMap`/`BTreeMap` 行为错乱。
- **复用成熟库 vs 重造轮子**：手写版利于**学算法**，工业版（pubgrub）利于**用得对**（回溯、冲突解释、久经考验）。我们两者都留，让对照成为教材。

## 7. 领域知识
- **pubgrub** 是 Dart `pub` 的 PubGrub 算法的 Rust 实现，被 `uv`、`cargo`（部分）等采用。它做**冲突驱动的回溯**：遇到冲突学习出"不兼容子句"，跳着回溯，既快又能给出**可读的冲突解释**。
- **可避免冲突 vs 真冲突**：贪心一旦选错版本就卡死；回溯能换条路。只有当**所有**版本组合都不相容时，pubgrub 才报真正的"无解"。

## 8. 软件设计理念
- **适配器模式**：我们的 `PackageIndex` ↔ pubgrub 的 `DependencyProvider`，靠 `OfflineDependencyProvider` + `constraint_to_range` 这层适配桥接，互不侵入。
- **可平替的接口**：`resolve` 与 `resolve_pubgrub` 签名一致（`Result<Map, ResolveError>`），于是上层可以**无痛切换**实现——下一步就这么做。

## 9. 小测验（自测）
1. 手写贪心求解器在 `BACKTRACK_SAMPLE` 上为什么失败，pubgrub 为什么成功？
2. 为什么 `Version` 要手写 `Hash` 而不能 `#[derive(Hash)]`？
3. `constraint_to_range` 这层在架构上叫什么？它解决了什么问题？
4. 为什么把手写求解器保留下来，而不是删掉？

## 10. 参考答案
1. 贪心先把 A 选成最高的 2.0，撞上 B 的 `A < 2.0` 就报冲突、且不回头；pubgrub 回溯，改选 A=1.0 即满足所有约束。
2. 我们的 `Eq` 视 `1.0 == 1.0.0`（去尾零相等），而 `derive(Hash)` 会按 `parts` 原样哈希，导致相等的值哈希不同、违反 Hash/Eq 契约；故手写去尾零的一致 `Hash`。
3. **适配器（防腐层）**。它把我们内部的 `Constraint` 翻译成外部库的 `Ranges`，让两套模型对接而互不污染。
4. 手写版是**教材**（讲清求解原理），工业版是**生产**（回溯、健壮）；保留两者可对照、可测试一致性，教学价值高。

## 11. 下一步预告
Step 23：把 CLI 的 `lock` / `install` 切换到 `resolve_pubgrub`，让 uvr **默认用工业级回溯求解器**；并验证联网求解、安装等端到端流程照常工作。
