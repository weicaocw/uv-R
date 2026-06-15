# Step 24：多仓库索引合并（解决跨仓库依赖）

> 模块：H 多仓库 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-24-multi-repo.md`）｜ 测试：✅ 30 passed ｜ 上一步：[Step 23](step-23-switch-to-pubgrub.md)

## 0. 一句话目标
把多个仓库的 `PACKAGES` 合并成一个索引，让"A 在甲仓库、却依赖乙仓库的 B"这类**跨仓库依赖**也能解出；并让每个包**记住自己来自哪个仓库**（安装时要据此下载）。

## 1. 前置回顾
单仓库时，r-universe 上的包常依赖 CRAN 上的包 → 当前会 `NotFound`/无解。本步引入**多仓库合并**，补上这个真实缺口——这也是迈向"能和 pak 同台 benchmark"的一步（pak 本就跨多仓库）。

## 2. 先写测试（TDD·红）
```rust
#[test]
fn resolves_across_merged_repos() {
    // A 在 r1、依赖 B；B 在 r2。单仓库解不全，合并后能解。
    let mut idx = PackageIndex::from_repo("Package: A\nVersion: 1.0\nImports: B\n", "https://r1");
    idx.merge(PackageIndex::from_repo("Package: B\nVersion: 1.0\n", "https://r2"));
    let sol = resolve_pubgrub(&idx, "A").unwrap();
    assert_eq!(sol["B"], Version::parse("1.0").unwrap());
}
```
（`from_repo` / `merge` / `Package.repo` 尚不存在时编译失败。）

## 3. 实现到通过（TDD·绿）
- `Package` 新增字段 `pub repo: String`（来源仓库基址）。
- `PackageIndex::from_repo(text, repo)`：解析并把每个包标记为来自 `repo`；`from_packages_file` = `from_repo(text, "")`。
- `PackageIndex::merge(other)`：把另一个索引的包**移动**并入（同名包各版本累加）。
```rust
pub fn merge(&mut self, other: PackageIndex) {
    for (name, pkgs) in other.by_name {
        self.by_name.entry(name).or_default().extend(pkgs);
    }
}
```
`cargo test` → ✅ 30 passed（跨仓库求解 + 来源追踪两个新用例）。

## 4. 改了哪些文件 / 加了什么
- `src/metadata.rs`：`Package` 加 `repo` 字段；`from_record` 接受 `repo`；新增 `from_repo` 与 `merge`。
- `src/resolver.rs`：新增 `merged_packages_keep_their_repo`、`resolves_across_merged_repos` 两个测试。

## 5. 学到的语法 / 技巧
- **给结构体加字段并贯通构造路径**：`Package` 多了 `repo`，构造它的 `from_record` 也随之加参数。
- **移动语义合并**：`for (name, pkgs) in other.by_name` 把 `other` 的内容**移动**进来（`other` 被消费），零克隆。
- **`entry(name).or_default().extend(pkgs)`**：把一批版本并入某包名下。

## 6. 语言设计巧思
- **数据自带来源（provenance）**：让 `Package` 记住 `repo`，下游"下载"才知道去哪个仓库取 tarball。把"未来需要的信息"在数据里带好，胜过到处传参数。
- **移动而非克隆**：`merge` 消费 `other`、移动其 `Vec`，Rust 的所有权让这种"零拷贝合并"既安全又显式。

## 7. 领域知识
- **真实 R 生态是多仓库的**：CRAN + Bioconductor + 各种 r-universe / drat。一个包的依赖常散落在不同仓库。
- **来源决定下载地址**：tarball URL = `<该包的仓库>/src/contrib/<name>_<ver>.tar.gz`——所以必须按"每个包各自的仓库"下载，而非单一仓库。
- pak 等成熟工具天然跨多仓库求解；uvr 补上这点，才谈得上公平对比。

## 8. 软件设计理念
- **用组合代替特例**：把"多仓库"建模成"多个单仓库索引的合并"，而不是到处写特判。
- **为下游需求预留信息**：在最早能拿到 `repo` 的地方（抓取/解析）就记下来，避免后续"丢了来源、无从下载"。

## 9. 小测验（自测）
1. 为什么 `Package` 要记 `repo`？谁会用到它？
2. `merge` 为什么能"零克隆"？
3. `from_packages_file` 和 `from_repo` 是什么关系？
4. 多仓库为什么是"能和 pak 公平 benchmark"的前提之一？

## 10. 参考答案
1. 因为安装时要按"包各自的来源仓库"拼出 tarball 下载地址；下载逻辑（模块 E / Step 25）会用到它。
2. `merge(other)` 按值消费 `other`，`for (name, pkgs) in other.by_name` 把内部 `Vec` **移动**进来，无需复制。
3. `from_packages_file(text)` 就是 `from_repo(text, "")`——即"不记来源"的特例。
4. 因为真实依赖跨多仓库；只有支持多仓库，uvr 才能像 pak 那样解出真实世界的依赖闭包，对比才有意义。

## 11. 下一步预告
Step 25：CLI 接受**多个 `--repo`**；`lock`/`install` 抓取并合并多个仓库后求解；安装时用**每个包各自的仓库**下载。现场演示一个跨 r-universe 仓库的依赖。
