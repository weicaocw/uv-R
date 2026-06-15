# Step 26：本地缓存模块（暖缓存的基础）

> 模块：I 缓存 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-26-cache.md`）｜ 测试：✅ 33 passed ｜ 上一步：[Step 25](step-25-cli-multi-repo.md)

## 0. 一句话目标
做一个**本地缓存**：按 URL 把抓到的文本（如 `PACKAGES`）存到缓存目录，重复使用时直接读盘，走"暖缓存"。

## 1. 前置回顾
现在每次 `lock` 都重新联网抓 `PACKAGES`。但同一个仓库短时间内反复抓很浪费。缓存让**第二次起免网络**——这正是 uv 招牌 benchmark 的关键轴（warm cache），也是 uvr 要"敢和 pak 端到端比"的必备能力（pak 也缓存元数据）。

## 2. 先写测试（TDD·红 → 绿）
```rust
#[test]
fn write_then_read_round_trips() {
    let dir = std::env::temp_dir().join("uvr_cache_unit_test");
    write(&dir, "u://k", "hello");
    assert_eq!(read(&dir, "u://k").as_deref(), Some("hello"));
    assert_eq!(read(&dir, "u://missing"), None);   // 未命中
}
```

## 3. 实现到通过（TDD·绿）
```rust
/// URL → 确定性、安全的缓存文件路径（非字母数字转下划线）。
pub fn cache_path(cache_dir: &Path, url: &str) -> PathBuf {
    let key: String = url.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    cache_dir.join(key)
}
pub fn read(cache_dir: &Path, url: &str) -> Option<String> {
    std::fs::read_to_string(cache_path(cache_dir, url)).ok()
}
pub fn write(cache_dir: &Path, url: &str, content: &str) {
    let _ = std::fs::create_dir_all(cache_dir);   // 失败静默忽略
    let _ = std::fs::write(cache_path(cache_dir, url), content);
}
```
`cargo test` → ✅ 33 passed。

## 4. 改了哪些文件 / 加了什么
- 新增 `src/cache.rs`：`cache_path` / `read` / `write` + 2 个测试。
- `src/lib.rs`：新增 `pub mod cache;`。

## 5. 学到的语法 / 技巧
- **`char` 映射造安全文件名**：`url.chars().map(|c| …).collect::<String>()` 把 URL 里的 `:/.` 等转成 `_`。
- **`std::fs`**：`read_to_string`（读）、`write`（写）、`create_dir_all`（建目录）。
- **`Option::ok()`**：把 `Result` 转 `Option`——读失败（含文件不存在）即视为"未命中"。
- **`let _ = …`**：显式忽略一个 `Result`（这里：缓存写失败不该让程序出错）。

## 6. 语言设计巧思
- **缓存是优化、不是正确性来源**：`write` 失败静默忽略（`let _ =`）——缓存写不进去顶多慢一点，绝不能让主流程崩。这是"优雅降级（graceful degradation）"。
- **确定性键**：同一个 URL 永远映射到同一个文件路径，缓存才命中得了。

## 7. 领域知识
- **元数据缓存**：pak 等工具都缓存仓库元数据，避免反复下载。`PACKAGES` 会更新，真实工具有 TTL / 刷新策略；我们先用最简单的"存在即用"（教学够用，后续可加刷新）。
- **暖缓存 = 性能关键**：日常反复 `lock`/`install` 时，命中缓存能省掉绝大部分网络时间——这是 uv 数量级提速的主要来源之一。

## 8. 软件设计理念
- **缓存层与抓取层分离**：`cache` 只管"存/取"，`fetch` 只管"联网"，下一步把两者组合成"先查缓存、未命中再抓"。单一职责，便于各自测试。
- **优雅降级**：外部资源（磁盘/网络）不可靠时，设计成"能用则用、不能用不致命"。

## 9. 小测验（自测）
1. 为什么 `write` 失败要静默忽略，而不是返回错误中断？
2. `cache_path` 为什么要把 URL 里的非字母数字字符替换掉？
3. `read` 用 `.ok()` 把 `Result` 转成 `Option`，"未命中"和"读出错"被如何对待？
4. 暖缓存为什么对"与 pak 公平 benchmark"很重要？

## 10. 参考答案
1. 缓存只是性能优化，写不进去顶多下次还得联网，不影响正确性；让它中断主流程是把"优化"变成了"故障点"。
2. URL 含 `:/.?` 等不适合做文件名的字符；替换成 `_` 得到一个合法、确定的文件名。
3. 两者都被当作"无缓存"（返回 `None`）——读不到就重新抓，简单且安全。
4. pak 默认缓存元数据；若 uvr 每次冷抓而 pak 用暖缓存，比较就不公平。uvr 也缓存后，才能在"暖缓存"这条 uv 招牌轴上同台对比。

## 11. 下一步预告
Step 27：把缓存接进 `fetch`（`get_text_cached`：先查缓存、未命中再抓并写回），CLI 用一个缓存目录；现场演示**冷抓 vs 暖缓存**的耗时差，并扩展 benchmark。
