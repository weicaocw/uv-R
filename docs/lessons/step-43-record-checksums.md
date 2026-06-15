# Step 43：记录校验和——lockfile v3

> 模块：R 完整性 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-43-record-checksums.md`）｜ 产物：`metadata.rs`（`Package.hash`）+ `lockfile.rs`（v3）+ `commands.rs`（`InstallItem.hash` 贯通）｜ 上一步：[Step 42](step-42-topological-install-order.md)

## 0. 一句话目标
把每个包的**校验和**从仓库元数据里提取出来，记进 lockfile（v3）——为下一步"下载后校验完整性"做准备。

## 1. 前置回顾
v2 锁文件记了版本 + 来源仓库。但"从对的地方下了对的版本"不等于"下到的内容没被篡改 / 损坏"。仓库的 `PACKAGES` 其实带了每个包的校验和：r-universe / PPM 给 **SHA256**、CRAN 给 **MD5sum**。本步把它记下来。

## 2. "测试"先行（TDD 红）
```rust
// 元数据提取：SHA256 优先、MD5sum 回退、都没有则空
from_packages_file(有 SHA256 的包).hash == "sha256:deadbeef"
from_packages_file(有 MD5sum 的包).hash == "md5:cafebabe"
from_packages_file(没校验和的包).hash == ""
// lockfile v3 四列往返
render({pkgA:(1.2.0, https://r1, sha256:aa)}) -> "...\npkgA 1.2.0 https://r1 sha256:aa\n"; parse 回来一致
parse("pkgA 1.2.0 https://r1")  -> hash 空（v2 兼容）
parse("pkgA 1.2.0")             -> repo/hash 空（v1 兼容）
```

## 3. 实现到通过（TDD 绿）
- `metadata.rs`：`Package` 加 `hash` 字段；`from_record` 取 `SHA256`（加前缀 `sha256:`）否则 `MD5sum`（`md5:`）否则空。
- `lockfile.rs`：`Locked` 加 `hash`；render 按可用信息伸缩列数（4 列含校验和 / 3 列含仓库 / 2 列）；parse 按列数兼容 v1/v2/v3。头部 `v3`。
- `commands.rs`：`InstallItem` 加 `hash`；`with_repos`、`install_plan`（从索引取）、`sync_plan`（从锁文件取）一路贯通。
68 测试全绿。本机 `uvr lock --repo <r-universe> dotenv`：
```
# uvr lockfile v3
dotenv 1.0.3.9000 https://gaborcsardi.r-universe.dev sha256:941d25df…ded11
```

## 4. 改了哪些文件 / 加了什么
- `src/metadata.rs`：`Package.hash` + 提取逻辑 + 1 个测试。
- `src/lockfile.rs`：v3（`Locked.hash`、render/parse 四列、兼容 v1/v2）+ 测试。
- `src/commands.rs`：`InstallItem.hash` 贯通 `with_repos`/`install_plan`/`sync_plan`。

## 5. 学到的语法 / 技巧
- **`.map(...).or_else(|| ...).unwrap_or_default()` 的优先级链**：先试 SHA256，没有再试 MD5sum，都没有给空串。`or_else` 惰性求值（只在前者为 `None` 时才算后者）——表达"优先 A、回退 B、兜底默认"非常顺。
- **带算法前缀的自描述值**：把校验和存成 `sha256:…` / `md5:…` 而非裸 hex。值**自带元信息**，校验时一看前缀就知道用哪个算法，无需另存一列。这是"让数据自描述"的小技巧。
- **格式按列数伸缩 + 兼容**：render 视字段是否为空决定写几列，parse 用切片模式按列数还原。**一套位置格式平滑容纳三代**（v1/v2/v3）。
- **字段沿数据流贯通**：一个新属性（hash）要从"元数据 → 锁文件/索引 → 安装项"一路带下去。每经一层就在对应结构上加字段、在转换处赋值——机械但要无遗漏。

## 6. 设计巧思 / 方法论
- **先记录、后使用，分两步**：本步只**提取并记录**校验和，不做校验；下一步才下载后比对。把"采集数据"和"使用数据"拆开，每步小、可独立验证（本步的"绿"= 锁文件里真的出现了 sha256）。
- **元数据驱动**：校验和不是我们算的，是**仓库在 `PACKAGES` 里声明的**。我们只忠实记录与传递。信任链从仓库的元数据开始——这也是为什么记下**来源仓库**（v2）是记**校验和**（v3）的前提。

## 7. 领域知识（R / 供应链安全）
- **R 仓库的校验和**：CRAN 的 `PACKAGES` 历史上给 `MD5sum`；r-universe、Posit PPM 等现代仓库给更强的 `SHA256`。记录它，就能在下载后验证"拿到的字节正是仓库登记的那份"。
- **为什么重要**：防的是**传输损坏**与**篡改**（中间人 / 被污染的镜像）。锁住版本（防漂移）+ 锁住校验和（防篡改）= 可复现**且**可信。`Cargo.lock` 记 checksum、`package-lock.json` 记 integrity，都是同一动机。
- **SHA256 vs MD5**：MD5 已不抗碰撞，仅适合查"意外损坏"；SHA256 抗篡改。所以我们**优先** SHA256。

## 8. 软件设计理念
- **可扩展的格式预留了今天**：上一模块把"锁定项"做成结构体而非裸版本，正是为今天加 `hash` 留的口子——加个字段、render/parse 多一列，调用方几乎无感。**昨天的抽象投资，今天免费兑现**。

## 9. 小测验（自测）
1. 为什么把校验和存成 `sha256:…` 这种带前缀的形式，而不是裸 hex？
2. lockfile 的 render 如何用一套位置格式同时支持 v1/v2/v3？parse 又如何还原？
3. 为什么本步只记录校验和、不做校验？这样拆分有什么好处？
4. 校验和是 uvr 自己算的吗？它从哪来？这对"信任"意味着什么？

## 10. 参考答案
1. 带前缀让值**自描述**：校验时一看 `sha256:` / `md5:` 就知道用哪个算法，无需另存算法列；也便于将来支持更多算法。
2. render 视字段是否为空决定写 2/3/4 列（有 hash 写四列、有 repo 写三列、否则两列）；parse 用切片模式按**列数**还原（2=v1、3=v2、4=v3）。位置 + 列数即版本。
3. 拆分让每步小而可验证：本步的"绿"是"锁文件里确实出现了 sha256"；下一步才接入真正的下载后比对。先采集、后使用，互不耦合、各自可测。
4. 不是 uvr 算的，是**仓库在 `PACKAGES` 里声明**的。我们忠实记录传递。信任链始于仓库元数据——所以"从哪个仓库取"（来源）与"内容对不对"（校验和）要一起锁。

## 11. 下一步预告
Step 44：接入 `sha2`，**下载后校验**——算出 tarball 的 SHA256 与锁文件比对，不符就报错拒装；演示正常通过与"篡改被发现"。然后发布 v0.12。
