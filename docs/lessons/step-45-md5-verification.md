# Step 45：补上 MD5 校验——兼容 CRAN

> 模块：S 完整性·补全 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-45-md5-verification.md`）｜ 产物：`Cargo.toml`（`md-5`）+ `install.rs`（`verify_hash` 按算法分派）｜ 上一步：[Step 44](step-44-verify-checksums.md)

## 0. 一句话目标
让完整性校验也支持 **MD5**，从而覆盖 CRAN 直连仓库（它在 `PACKAGES` 里给的是 `MD5sum`）。

## 1. 前置回顾
Step 44 实现了 SHA256 校验（r-universe / PPM 提供）。但 **CRAN** 的 `src/contrib/PACKAGES` 给的是 `MD5sum`——上一步我们把它记成 `md5:…` 却**跳过了**校验。本步把这块补上，整个整合链对两类仓库都生效。

## 2. "测试"先行（TDD 红）
扩展校验函数的用例，sha256 与 md5 并列：
```rust
verify_hash(文件, "sha256:<对>") == Ok      verify_hash(文件, "sha256:0000") == Err
verify_hash(文件, "md5:<对>")    == Ok      verify_hash(文件, "md5:0000")    == Err
verify_hash(文件, "")            == Ok      // 空 → 跳过
verify_hash(文件, "crc32:abcd")  == Ok      // 未知算法 → 跳过
```
端到端（CRAN 真包 `praise`）：
```
$ uvr lock --repo https://cran.r-project.org praise > uvr.lock
$ cat uvr.lock
# uvr lockfile v3
praise 1.0.0 https://cran.r-project.org md5:9318724cec0454884b5f762bee2da6a1
$ uvr sync --lib ./r-lib          # ← 用 md5 校验后安装
synced praise 1.0.0
# 篡改后再 sync：
同步失败 / sync failed: praise 1.0.0: md5 校验和不符 …            # ← 被拦下
```

## 3. 实现到通过（TDD 绿）
- `Cargo.toml`：加依赖 `md-5 = "0.10"`（RustCrypto；注意 crate 名带连字符 `md-5`，但导入用 `md5`）。
- `install.rs`：`verify_hash` 从"只认 sha256"改为**按前缀分派**：`split_once(':')` 拆出 `algo` 与 `want`，`match algo { "sha256" => …, "md5" => …, _ => 跳过 }`。抽出 `to_hex` 小助手复用。
69 测试全绿（校验测试现并测 sha256 与 md5）；CRAN `praise` 演示如上。

## 4. 改了哪些文件 / 加了什么
- `Cargo.toml`：`md-5` 依赖。
- `src/install.rs`：`to_hex` 助手；`verify_hash` 按算法前缀分派（sha256 / md5）；测试扩展到两种算法 + 未知算法跳过。

## 5. 学到的语法 / 技巧
- **`split_once(':')` 拆前缀**：把 `"md5:abcd"` 一刀两段成 `("md5","abcd")`，比 `strip_prefix` 一个个试更适合"多算法分派"。返回 `Option`，配 let-else 跳过无前缀的情况。
- **`match` 做算法分派**：一个 `match algo` 把"哪个算法"映射到"哪套计算"，未知算法落到 `_ => Ok(())`（跳过）。**用 match 表达"有限集合的分派"**，清晰且编译器帮你查遗漏。
- **抽出 `to_hex` 复用**：sha256 与 md5 的结果都要转十六进制，抽成一个小函数，两处共用——发现重复就抽。
- **crate 名 vs 导入名**：`md-5` 是包名（Cargo.toml），`md5` 是它的库名（`use md5::Md5`）。Rust 里两者可不同，知道这点免得困惑。

## 6. 设计巧思 / 方法论
- **可扩展的分派点**：`verify_hash` 现在是一个清晰的"算法 → 计算"分派点。将来要支持 `sha512`、`sha1`，只是 `match` 里多一臂——**把"会增长的维度"收敛到一处**，扩展成本最低。
- **对称地完成一件事**：上一步留了"md5 暂跳过"的口子，本步补齐，让"记录"与"校验"对两类仓库都对称生效。**把开的口子收掉**，功能才算真正完整、文档才不必长期挂着 caveat。

## 7. 领域知识（R / 校验和）
- **CRAN 直连只给 MD5sum**：CRAN 的 `src/contrib/PACKAGES` 历史上只登记 `MD5sum`。要对**直连 CRAN** 的包做完整性校验，就必须支持 MD5。（经 Posit PPM 等镜像访问 CRAN 时往往能拿到 SHA256。）
- **MD5 的定位**：MD5 不抗蓄意碰撞，但足以发现**意外损坏**（传输截断、磁盘坏块）和无针对性的篡改。在仓库只提供 MD5 时，"用 MD5 校验"远胜"不校验"。能拿到 SHA256 时我们仍**优先** SHA256（见 Step 43 的提取顺序）。

## 8. 软件设计理念
- **务实的安全分级**：理想是处处 SHA256，但现实里 CRAN 给 MD5。与其因为"MD5 不够强"而**不校验**，不如"有什么用什么"——用仓库提供的最强校验和，并在能选时选更强的。**完美不该是良好的敌人**。
- **收口而非堆叠**：本步没有新增功能维度，而是把上一步的"未完成分支"补全。健康的迭代既要开拓，也要**回头收口**，避免技术债与文档 caveat 越积越多。

## 9. 小测验（自测）
1. 为什么必须支持 MD5，而不能只用 SHA256？
2. `verify_hash` 用 `split_once(':')` + `match algo` 的结构，相比"逐个 `strip_prefix` 试"有什么好处？
3. 仓库同时（理论上）给了 SHA256 和 MD5，uvr 会记录 / 校验哪个？为什么？
4. MD5 已知不抗碰撞，为什么用它校验仍然有意义？

## 10. 参考答案
1. CRAN 直连的 `PACKAGES` 只提供 `MD5sum`；不支持 MD5 就无法校验直连 CRAN 的包。要让完整性校验覆盖主流仓库，必须支持 MD5。
2. `split_once + match` 是"一次拆分、按算法分派"，新增算法只在 `match` 加一臂、未知算法统一落到跳过；逐个 `strip_prefix` 试则随算法增多越来越啰嗦，且容易漏掉"未知算法"的统一处理。
3. 记录时**优先 SHA256**（Step 43 的提取顺序：先 `SHA256` 后 `MD5sum`）；校验时按记录下来的前缀算。优先更强的哈希。
4. MD5 足以发现**意外损坏**与非针对性篡改；在仓库只给 MD5 时，"用 MD5 校验"远胜"完全不校验"。务实地用所能得到的最强校验和。

## 11. 下一步预告
模块 S 完成、发布 **v0.13**：完整性校验对 SHA256（r-universe/PPM）与 MD5（CRAN）双覆盖。uvr 至此形态完整：解析 → 锁（版本+来源+校验和）→ 同步（自包含、校验、拓扑序、并行下载）→ 装（项目本地库、选定的 R）。后续可继续：签名验证（真实性）、按拓扑层并行安装、以及换到支持 binary 的环境补齐模块 J。
