# Step 27：把缓存接进抓取与下载（暖缓存提速 ~94×）

> 模块：I 缓存 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-27-warm-cache.md`）｜ 测试：✅ 33 passed + 冷/暖实测 ｜ 上一步：[Step 26](step-26-cache.md)

## 0. 一句话目标
把 Step 26 的缓存接进 `fetch` 与 `install`：抓 `PACKAGES`"先查缓存、未命中再联网并写回"；下载 tarball"已存在则跳过"。重复操作走**暖缓存**、免联网。

## 1. 前置回顾
Step 26 有了缓存模块（存/取）。本步把它**织进**抓取与下载的边界，让第二次起的 `lock`/`install` 跳过网络。

## 2. "测试"：回归 + 冷/暖实测
既有 33 个测试在改动后仍全绿（回归保护）。并**实测**冷抓 vs 暖缓存的耗时差。

## 3. 实现到通过（绿）
```rust
// fetch.rs：带缓存的抓取
pub fn get_text_cached(url: &str, cache_dir: &Path) -> Result<String, FetchError> {
    if let Some(cached) = cache::read(cache_dir, url) { return Ok(cached); } // 命中
    let text = get_text(url)?;            // 未命中 → 联网
    cache::write(cache_dir, url, &text);  // 写回缓存
    Ok(text)
}

// install.rs：tarball 已在则跳过下载
pub fn download(url: &str, dest: &Path) -> Result<(), String> {
    if dest.exists() { return Ok(()); }   // 已缓存
    /* ... 下载 ... */
}
```
`main` 用 `.uvr-cache/`（`meta/` 存 PACKAGES、`tarballs/` 存包）。

`cargo test` → ✅ 33 passed。**冷 vs 暖（`uvr lock --repo … jsonlite`，release）**：
```
cold (无缓存·联网抓): 640.1 ms
warm (暖缓存·免联网):   6.8 ms     ← ~94× 提速
```

## 4. 改了哪些文件 / 加了什么
- `src/fetch.rs`：新增 `get_text_cached`（缓存旁路）。
- `src/install.rs`：`download` 在目标已存在时跳过。
- `src/main.rs`：抓取走 `get_text_cached`，缓存目录 `.uvr-cache/{meta,tarballs}`。
- `scripts/bench.sh`：新增 cold vs warm 一行；`.gitignore` 忽略 `/.uvr-cache`。

## 5. 学到的语法 / 技巧
- **缓存旁路（cache-aside）**：`if let Some(c) = cache::read(...) { return Ok(c) }` —— 先查、命中即返回，未命中再做真正的工作并写回。
- **`if dest.exists() { return ... }`**：早退跳过已完成的工作。
- **把缓存目录作为参数传入**：`get_text_cached(url, cache_dir)`——纯粹、可控、可测。

## 6. 语言设计巧思
- **缓存是横切关注点，在边界处织入**：`fetch`/`install` 在最外层加缓存，求解、解析等**核心逻辑完全不知道缓存的存在**。关注点分离让缓存可加可去、不污染核心。
- **暖缓存是数量级提速的来源**：冷抓 640ms 几乎全是网络往返；暖缓存 6.8ms 只是读盘 + 解析。这正是 uv "快几十上百倍"的主因之一。

## 7. 领域知识
- **元数据缓存 + 下载缓存**：仓库 `PACKAGES` 与包 tarball 都值得缓存。pak 等工具默认缓存元数据；uvr 跟进后，"暖缓存"这条 uv 招牌轴才能与之同台对比。
- **失效策略（本步从简）**：真实工具会按 TTL / etag 刷新元数据；我们先用"存在即用"，教学够用，可后续加刷新。

## 8. 软件设计理念
- **装饰 / 缓存旁路**：`get_text_cached` 是 `get_text` 的"带缓存装饰版"，对调用方透明地提速。
- **优雅降级**：缓存读写失败都不致命（读失败=未命中重抓；写失败=下次再抓）。

## 9. 小测验（自测）
1. "缓存旁路（cache-aside）"的三步是什么？
2. 为什么缓存逻辑放在 `fetch`/`install` 边界，而不是塞进求解核心？
3. 冷 640ms、暖 6.8ms，省下的主要是什么时间？
4. 为什么暖缓存对"与 pak 公平对比"重要？

## 10. 参考答案
1. 先查缓存；命中即返回；未命中则做真正的工作（联网/下载）并把结果写回缓存。
2. 缓存是横切关注点；放在边界处可让核心逻辑与"数据从哪来"解耦，缓存可独立增删、单独测试，核心不受污染。
3. 主要省下**网络往返**时间（冷抓要联网取 PACKAGES，暖缓存只读本地盘 + 解析）。
4. pak 默认缓存元数据；若 uvr 每次冷抓而 pak 暖缓存，对比不公平。uvr 也缓存后，才能在暖缓存这条轴上同条件比较。

## 11. 下一步预告
模块 I（缓存）收官，开 PR（v0.5）。
下一模块 **J：binary 包**——优先选用预编译的 binary 包（免 `R CMD INSTALL` 编译），让"安装"与 pak 处于**同一条件**，从而能做公平的端到端安装对比。
