# Step 41：让 `sync` 自包含——`--repo` 变可选

> 模块：P lockfile v2 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-41-self-contained-sync.md`）｜ 产物：`src/main.rs`（`sync` 去掉 `--repo` 必填）｜ 上一步：[Step 40](step-40-lockfile-v2.md)

## 0. 一句话目标
v2 锁文件已自带来源仓库，于是 `uvr sync` 可以**不带 `--repo`** 一键还原。

## 1. 前置回顾
Step 40 让锁文件记下了每个包的仓库，`sync_plan` 也会优先用它。但 CLI 还硬性要求 `--repo`（`repos.is_empty()` 就报错）。本步把这道限制去掉。

## 2. "测试"：端到端演示（show, don't tell）
本机实跑，全程**不带 `--repo`**：
```
$ uvr lock --repo <r-universe> dotenv > uvr.lock
$ cat uvr.lock
# uvr lockfile v2
dotenv 1.0.3.9000 https://gaborcsardi.r-universe.dev

$ uvr sync --lib ./r-lib            # ← 没有 --repo！
→ 使用 R / using R 4.5.2: /usr/local/bin/R
synced dotenv 1.0.3.9000
→ 已按 uvr.lock 还原到项目本地库 / restored from uvr.lock into: ./r-lib
```
全量 62 测试（+3 ignored）绿（CLI 改动由演示覆盖，sync_plan 自包含路径已在 Step 40 单测）。

## 3. 实现到通过（TDD 绿）
就删掉一段：
```rust
// 删除：
if repos.is_empty() {
    eprintln!("sync 需要 --repo ...");
    return usage();
}
```
`--repo` 为空时 `fetch_sources` 返回空来源；`sync_from_lock` → `sync_plan` 用锁文件里的仓库。若锁文件是旧 v1（无仓库）且没给 `--repo`，`sync_plan` 会清楚报"找不到来源"。

## 4. 改了哪些文件 / 加了什么
- `src/main.rs`：`sync` 去掉 `--repo` 必填校验，加注释说明 v2 自包含、v1 才需 `--repo` 兜底。

## 5. 学到的语法 / 技巧
- **删代码也是进步**：本步的"实现"是**移除**一道限制。前面把数据格式（v2）和回退逻辑铺好，到这里界面就能顺势简化——好的底层设计让上层"自然变简单"。
- **空集合的良性传播**：`fetch_sources(&[])` 返回空 `Vec`、`build_index(&[])` 得空索引——空输入一路安全流过，无需特判。设计函数时让"空"是合法输入，能省掉大量边界 if。
- **错误延后到该报的地方**：不在 CLI 早早拦 `--repo`，而是让缺来源的情况一路走到 `sync_plan`，在"真正缺了来源"时才按包名报错。报错点离问题越近，信息越准。

## 6. 设计巧思 / 方法论
- **自包含的锁文件 = 更少的心智负担**：用户不必记"当初从哪个仓库锁的"。`git clone` 下来一个带 `uvr.lock` 的项目，一句 `uvr sync` 就能还原——这正是 `cargo build`（读 `Cargo.lock`）、`npm ci`（读 `package-lock.json`）的体验。
- **两步走收尾**：Step 40 升级格式+逻辑（保持全绿、不改用户用法），Step 41 才简化界面。**先让新路径可用，再让它成为默认**——平滑、可回滚。

## 7. 领域知识（包管理 / 可复现）
- **可复现的黄金路径**：作者 `lock` 一次 → 提交 `uvr.lock` → 任何人 `clone` 后 `uvr sync` 还原**完全相同**的依赖（版本 + 来源）。这是现代包管理器的标配工作流，uvr 现在也有了。
- **v1 仍受支持**：老的 v1 锁文件 + `--repo` 照样能 `sync`。新功能不淘汰老用户——向后兼容是工具可信赖的基础。

## 8. 软件设计理念
- **简化是功能**：把"必须传 `--repo`"变成"通常不用传"，没加任何新代码路径，反而删了校验。**减少必填参数**直接降低使用摩擦——少即是多。
- **分层回报**：版本模型（Step 01）、缓存（I）、多仓库（H）、lockfile（v1→v2）一层层铺下来，到这一步"自包含 sync"几乎是免费长出来的。前期在抽象上的投资，在后期以"新功能很便宜"的形式回报。

## 9. 小测验（自测）
1. 为什么本步几乎只是"删代码"，却是一个有价值的功能改进？
2. `--repo` 为空时，整条 `sync` 链路是怎么安全跑下去的？
3. 锁文件是旧 v1、又没传 `--repo`，会发生什么？这种处理好在哪？
4. "自包含锁文件"对从 `git clone` 一个项目的新人意味着什么？

## 10. 参考答案
1. 因为前面（v2 格式 + sync_plan 优先用锁文件仓库）已把能力铺好；本步移除一道不再必要的限制，就把"自包含 sync"这个真实价值释放出来。改进未必都靠加代码，**移除限制**同样是进步。
2. `--repo` 为空 → `fetch_sources` 返回空来源 → `build_index` 得空索引 → `sync_plan` 改用**锁文件里记的仓库**拼地址。空集合一路良性传播，无需特判。
3. `sync_plan` 在该包既无锁文件仓库、又无 `--repo` 来源时，按**包名**报"找不到来源"。报错点贴近真正缺失处，信息精确，用户知道要么补 `--repo`、要么用 v2 锁文件。
4. 新人 `clone` 后无需知道"原作者从哪些仓库装的"，一句 `uvr sync` 即按 `uvr.lock` 还原相同版本与来源——零额外知识、可复现。

## 11. 下一步预告
模块 P 完成、发布 **v0.10**。后续可继续：**拓扑序安装**（依赖先于依赖者，并为"多包并行安装"铺路）、锁文件再记**校验和**（sha256，验证完整性）、以及换到支持 binary 的环境补齐模块 J。
