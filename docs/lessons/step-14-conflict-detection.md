# Step 14：冲突检测（`Result` + 错误枚举）

> 模块：D 依赖求解 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-14-conflict-detection.md`）｜ 测试：✅ 18 passed ｜ 上一步：[Step 13](step-13-transitive-resolution.md)

## 0. 一句话目标
让求解失败时**带上原因**：把 `resolve` 的返回从 `Option` 升级为 `Result<…, ResolveError>`，区分"找不到包 / 无满足版本 / 版本冲突"。

## 1. 前置回顾
Step 13 的 `resolve` 返回 `Option`——失败时只能说"没解出来"，说不清为什么。真实工具要能告诉用户**哪里**出了问题。本步引入错误类型，并补上 Step 13 故意略过的**版本冲突**检测。

## 2. 先写测试（TDD·红）
```rust
#[test]
fn detects_version_conflict() {
    // low 要 pkgA < 1.1.0，high 要 pkgA >= 1.2.0 —— 不可兼容
    let idx = PackageIndex::from_packages_file(CONFLICT_SAMPLE);
    assert_eq!(resolve(&idx, "root"), Err(ResolveError::Conflict("pkgA".to_string())));
}
```
（`ResolveError` 与新签名尚不存在时编译失败；红从略。）

## 3. 实现到通过（TDD·绿）
```rust
#[derive(Debug, PartialEq, Eq)]
pub enum ResolveError {
    NotFound(String),       // 索引里没有这个包
    Unsatisfiable(String),  // 包在，但没有版本满足约束
    Conflict(String),       // 同一包被要求互不兼容的版本
}

pub fn resolve(index: &PackageIndex, root: &str)
    -> Result<BTreeMap<String, Version>, ResolveError>
{
    // ... 工作队列 ...
    if let Some(chosen) = resolved.get(&name) {
        if let Some(c) = &constraint && !c.matches(chosen) {
            return Err(ResolveError::Conflict(name));   // 已选版本不满足新约束 → 冲突
        }
        continue;
    }
    if index.versions_of(&name).is_empty() {
        return Err(ResolveError::NotFound(name));        // 包不存在
    }
    let pkg = best_match(index, &name, constraint.as_ref())
        .ok_or_else(|| ResolveError::Unsatisfiable(name.clone()))?; // 无满足版本
    // ...
}
```
`cargo test` → ✅ 18 passed（新增冲突、未找到两个用例）。

## 4. 改了哪些文件 / 加了什么
- `src/resolver.rs`：新增 `enum ResolveError`；`resolve` 返回类型改为 `Result`；加入"已解析则校验约束兼容"的冲突检测；新增 `detects_version_conflict`、`missing_dependency_is_not_found` 测试。

## 5. 学到的语法 / 技巧
- **`Result<T, E>`**：成功 `Ok(T)` / 失败 `Err(E)`。比 `Option` 多带了"失败原因"。
- **带数据的枚举**：`enum ResolveError { NotFound(String), ... }`——每个分支可携带相关信息（哪个包）。
- **`#[derive(PartialEq, Eq)]` 之于错误**：让测试能用 `assert_eq!(result, Err(...))` 直接比对错误。
- **`ok_or_else(|| ...)`**：把 `Option` 转成 `Result`；用**闭包**惰性构造错误，仅在确实是 `None` 时才构造（避免无谓的 `clone`）。
- **let-chain `if let Some(c) = &constraint && !c.matches(chosen)`**：一行表达"有约束且不满足"。

## 6. 语言设计巧思
- **`Option` 还是 `Result`？**：仅"有/无"用 `Option`；"成功/失败且想知道为什么"用 `Result`。Rust 用类型把这个区别摆上台面。
- **错误即数据**：把失败模式建模成一个**枚举**，调用方可 `match` 穷尽处理每种错误——比"抛异常/返回错误码"更安全、更自描述。
- **`ok_or_else` vs `ok_or`**：后者会**总是**先构造错误值（即便是 `Ok`）；带分配（`clone`）时浪费，clippy 的 `or_fun_call` 会提醒改用惰性的 `ok_or_else`。

## 7. 领域知识
- **版本冲突**是依赖求解的核心难题：依赖图里两条路径可能要求同一个包的不兼容版本。
- **我们的简化**：贪心选"最高满足版本"，发现不兼容就报 `Conflict`，**不回溯**。因此它可能把"换个版本本可共存"的情况也报成冲突。
- **pubgrub** 等工业求解器会**回溯**并给出可读的冲突解释——这是日后用真实库替换本教学实现时的升级点。

## 8. 软件设计理念
- **用类型表达失败模式**：`ResolveError` 让"会怎样失败"成为 API 的一部分，调用方被编译器要求处理。
- **诚实标注局限**：在文档与注释里写明"不回溯""可能误报可避免冲突"，让使用者明确边界——这是工程诚信，也方便日后升级。

## 9. 小测验（自测）
1. `Option` 与 `Result` 的区别是什么？本步为何从前者换成后者？
2. `ResolveError::Conflict(String)` 里的 `String` 用来装什么？
3. 为什么用 `ok_or_else(|| ...)` 而不是 `ok_or(...)`？
4. 我们的冲突检测有什么已知局限？工业级求解器怎么做得更好？

## 10. 参考答案
1. `Option` 只表达"有/无"；`Result` 表达"成功/失败"并携带失败原因。本步需要告诉调用方"为什么解不出来"，所以用 `Result`。
2. 装出问题的**包名**，方便错误信息指明是哪个包冲突 / 缺失。
3. `ok_or` 会无条件先构造错误值（这里含 `clone`，即便结果是 `Ok` 也白构造）；`ok_or_else` 用闭包**惰性**构造，仅在 `None` 时才付出代价。
4. 它贪心、不回溯，可能把"换个版本本可共存"的情况误报为冲突；pubgrub 等会回溯尝试其它版本组合，并在真冲突时给出清晰解释。

## 11. 下一步预告
Step 15：把求解结果写成 **lockfile**（纯 std 的确定性文本格式）并能读回来（往返测试）。有了 lockfile，"可复现安装"就有了落点——模块 D 收官。
