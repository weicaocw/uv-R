# Step 40：lockfile v2——记录来源仓库（向后兼容 v1）

> 模块：P lockfile v2 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-40-lockfile-v2.md`）｜ 产物：`src/lockfile.rs`（`Locked` + v2 render/parse）+ `src/commands.rs`（`with_repos` + `sync_plan` 用锁文件仓库）｜ 上一步：[Step 39](step-39-jobs-flag.md)

## 0. 一句话目标
让 lockfile 多记一列**来源仓库**（v2），并保证仍能读懂旧的 v1 锁文件——为"`sync` 不再需要 `--repo`"铺路。

## 1. 前置回顾
v1 锁文件只有 `name version`，所以 `sync` 得靠 `--repo` 才知道去哪下载（见 Step 36）。如果锁文件**自己记下**每个包来自哪个仓库，`sync` 就能自包含。本步升级格式，下一步改 CLI。

## 2. "测试"先行（TDD 红）
```rust
// v2 三列往返（含仓库）
render({pkgA: (1.2.0, https://r1)}) -> "...\npkgA 1.2.0 https://r1\n"; parse 回来一致
// 无仓库退化为两列（不留尾随空格）
render({pkgA: (1.0.0, "")}) -> 含 "pkgA 1.0.0\n"，不含 "pkgA 1.0.0 "
// 兼容旧 v1：两列 → repo 为空
parse("pkgA 1.2.0") -> {pkgA: (1.2.0, "")}
// 非法：1 列或 4 列
parse("badline") == None;  parse("a 1.0 r extra") == None
```

## 3. 实现到通过（TDD 绿）
```rust
pub struct Locked { pub version: Version, pub repo: String }

pub fn render(res: &BTreeMap<String, Locked>) -> String { /* repo 空写两列、否则三列 */ }

pub fn parse(text: &str) -> Option<BTreeMap<String, Locked>> {
    // 按 split_whitespace 切；用切片模式匹配列数：
    match parts.as_slice() {
        [n, v]    => (n, v, ""),    // v1
        [n, v, r] => (n, v, r),     // v2
        _ => return None,           // 其它列数非法
    }
}
```
`commands.rs`：新增 `with_repos(index, resolved)` 给求解结果补仓库；`lock_from_sources` 渲染 v2；`sync_plan` 改吃 `Locked`——**优先用锁文件里的仓库**，为空（v1）才回退到 `--repo` 索引。62 测试全绿。

本机实跑 `uvr lock --repo <r-universe> dotenv`：
```
# uvr lockfile v2
dotenv 1.0.3.9000 https://gaborcsardi.r-universe.dev
```

## 4. 改了哪些文件 / 加了什么
- `src/lockfile.rs`：`Locked` 结构、v2 `render`/`parse`（兼容 v1）、4 个测试（v2 往返 / 两列退化 / v1 兼容 / 非法列数）。
- `src/commands.rs`：`with_repos` 助手、`lock_from_sources` 渲染 v2、`sync_plan` 用锁文件仓库 + v1 回退；更新 / 新增 3 个 sync 测试（含"无 `--repo` 自包含"）。

## 5. 学到的语法 / 技巧
- **切片模式匹配 `match parts.as_slice()`**：用 `[n, v]` / `[n, v, r]` 直接按**元素个数**分支并解构——比 `if parts.len()==2 {...}` 既安全又直观。这是 Rust 处理"按长度分情况"的利器。
- **结构体承载多字段**：从 `Version` 升级到 `Locked { version, repo }`。当一个值需要多于一个属性时，**造个有名字的结构体**胜过用元组 `(Version, String)`——字段有名、易扩展（将来还能加 hash、时间戳）。
- **`unwrap_or_default()`**：找不到仓库时给 `String::default()`（空串），简洁表达"缺省为空"。
- **向后兼容的解析**：同一个 `parse` 同时吃 v1 / v2，靠列数区分。**新版读得懂旧文件**是格式演进的基本功——别让升级把用户的旧锁文件作废。

## 6. 设计巧思 / 方法论
- **格式版本化**：头部注释从 `v1` 升到 `v2`，但解析按**结构**（列数）而非头部字符串来决定怎么读，于是混着 v1/v2 也不怕。版本号给人看，解析按形状走——稳。
- **渐进迁移**：先升级**数据格式与库逻辑**（本步，保持全绿），再改**用户界面**（下一步让 `--repo` 可选）。一次只动一层，每步可独立验证、可回滚。
- **优先 + 回退**：`sync_plan` 优先用锁文件的仓库，没有才回退到 `--repo`。**新能力不破坏旧用法**——拿着 v1 锁文件 + `--repo` 的老用户照样能跑。

## 7. 领域知识（包管理）
- **锁文件该记什么**：版本是底线；记下**来源**（仓库 / URL / 校验和）让还原**自包含**且可验证。`Cargo.lock` 记 registry + checksum，`package-lock.json` 记 resolved URL + integrity。uvr v2 先迈出"记来源仓库"这一步。
- **为什么 R 尤其需要记来源**：R 包可能来自 CRAN、Bioconductor、r-universe、GitHub……同名包不同源。只记版本不记来源，换台机器可能从**别的仓库**装到**不同的同名包**。记下来源消除这种歧义。

## 8. 软件设计理念
- **数据结构的演进要预留空间**：把"锁定项"做成结构体而非裸版本，今天加 `repo`，明天加 `sha256` 都只是加字段，不必重写签名。**为可预见的扩展留接口**，是降低未来改动成本的关键。

## 9. 小测验（自测）
1. `parse` 如何同时支持 v1（两列）和 v2（三列）？为什么按列数而不是按头部 `v1/v2` 字样判断？
2. 为什么把锁定项从 `Version` 升级成 `Locked` 结构体，而不是用元组 `(Version, String)`？
3. `sync_plan` 里"优先锁文件仓库、为空才回退 `--repo`"这个顺序，保护了哪类用户？
4. 锁文件只记版本、不记来源，在 R 生态里可能出什么问题？

## 10. 参考答案
1. `parse` 用切片模式匹配列数：两列按 v1（repo 空）、三列按 v2。按**结构**判断比信任头部字样更稳——即使头部写错或文件混排，也能正确解析每一行。
2. 结构体字段**有名字**（`version`/`repo`），可读性强、易扩展（再加 `sha256` 只是加字段），还能 `derive` 比较 / 克隆。元组 `(Version, String)` 字段靠位置、含义不清、扩展即破坏所有解构点。
3. 保护**老用户**：手里是 v1 锁文件（无仓库）+ 习惯传 `--repo` 的人。优先用锁文件、为空回退 `--repo`，让新格式不破坏旧用法（向后兼容）。
4. R 包可能来自 CRAN / Bioconductor / r-universe / GitHub 等多个源，存在同名不同源的包。只记版本，换机器可能从**别的仓库**装到**同名但不同**的包，破坏可复现。记来源仓库消除歧义。

## 11. 下一步预告
Step 41：让 `uvr sync` 的 `--repo` **变为可选**——v2 锁文件自包含时一句 `uvr sync` 即可还原；演示 lock→sync 全程不带 `--repo`。然后发布 v0.10。
