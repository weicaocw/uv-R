# Step 11：包索引 / 依赖图

> 模块：B 元数据 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-11-package-index.md`）｜ 测试：✅ 12 passed ｜ 上一步：[Step 10](step-10-dependency-fields.md)

## 0. 一句话目标
定义 `Package`（名字 + 版本 + 依赖），把整份 `PACKAGES` 解析成"**包名 → 各版本**"的索引，能按名字查依赖——这就是**依赖图**。

## 1. 前置回顾
Step 09 能解析 DCF 记录，Step 10 能解析依赖字段。本步把它们**组装**起来：每条记录变成一个 `Package`，所有包汇成一个可查询的 `PackageIndex`。它就是模块 D（依赖求解）的输入。

## 2. 先写测试（TDD·红）
```rust
#[test]
fn builds_index_with_deps() {
    let idx = PackageIndex::from_packages_file(SAMPLE);
    assert_eq!(idx.len(), 2);
    let a3 = &idx.versions_of("A3")[0];
    assert_eq!(a3.version, Version::parse("1.0.0").unwrap());
    assert_eq!(a3.depends.len(), 3); // R, xtable, pbapply
    assert!(idx.versions_of("nonexistent").is_empty());
}
```
实现 `Package` / `PackageIndex` 之前，编译失败（`E0433`）。（红从略。）

## 3. 实现到通过（TDD·绿）
```rust
#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub depends: Vec<Dependency>,
}

impl Package {
    fn from_record(rec: &Record) -> Option<Package> {
        let name = rec.get("Package")?.clone();
        let version = Version::parse(rec.get("Version")?)?;
        let mut depends = Vec::new();
        for field in ["Depends", "Imports", "LinkingTo"] {
            if let Some(value) = rec.get(field) {
                depends.extend(parse_dependencies(value));
            }
        }
        Some(Package { name, version, depends })
    }
}

#[derive(Debug, Default)]
pub struct PackageIndex {
    by_name: BTreeMap<String, Vec<Package>>,
}

impl PackageIndex {
    pub fn from_packages_file(input: &str) -> PackageIndex {
        let mut by_name: BTreeMap<String, Vec<Package>> = BTreeMap::new();
        for rec in parse(input) {
            if let Some(pkg) = Package::from_record(&rec) {
                by_name.entry(pkg.name.clone()).or_default().push(pkg);
            }
        }
        PackageIndex { by_name }
    }
    pub fn versions_of(&self, name: &str) -> &[Package] {
        self.by_name.get(name).map(Vec::as_slice).unwrap_or(&[])
    }
    pub fn len(&self) -> usize { self.by_name.len() }
    pub fn is_empty(&self) -> bool { self.by_name.is_empty() }
}
```
`cargo test` → ✅ 12 passed。

## 4. 改了哪些文件 / 加了什么
- `src/metadata.rs`：新增 `struct Package`、`Package::from_record`、`struct PackageIndex`、`from_packages_file` / `versions_of` / `len` / `is_empty`，以及 1 个索引测试。

## 5. 学到的语法 / 技巧
- **`?` 用在 `Option` 上**：`rec.get("Package")?` —— 没有该字段就让 `from_record` 直接返回 `None`，跳过这条记录。
- **`for field in ["Depends", "Imports", "LinkingTo"]`**：直接遍历数组字面量。
- **`Vec::extend`**：把一批依赖追加进 `depends`。
- **`BTreeMap::entry(k).or_default().push(v)`**：典型的"分组收集"——键不存在就插入默认（空 `Vec`），再 push。
- **`#[derive(Default)]`**：让 `PackageIndex` 有一个"空索引"的默认值（`entry`/`or_default` 等也依赖 `Default`）。
- **返回 `&[Package]` 空切片**：`...unwrap_or(&[])` 在查不到时返回空切片，调用方无需区分"无此包"与"有但空"。

## 6. 语言设计巧思
- **借用而非克隆**：`versions_of` 返回 `&[Package]`（借用索引内部数据），不复制——调用方按需读取，零拷贝。
- **clippy 的 `len_without_is_empty`**：一旦提供 `len()`，clippy 要求也提供 `is_empty()`（语义完整、避免 `len()==0` 的别扭写法）。这又是 lint 在引导我们写更地道的 API。

## 7. 领域知识
- **`PACKAGES` = 依赖图的原料**：一份文件里成百上千条记录，每条是一个包（含 `Depends`/`Imports`/`LinkingTo` 等依赖）。把它们索引起来，就能回答"X 依赖谁、X 有哪些版本"。
- **一个包名可能有多个版本**：所以索引值是 `Vec<Package>`（虽然单一 `PACKAGES` 通常每名一条，但跨仓库/快照会有多版本）。
- **安装依赖的合并**：`Depends + Imports + LinkingTo` 共同决定"装这个包还得装什么"；`Suggests` 是可选、通常不计入。
- 其中 `R (>= x)` 是对 R 本身的"伪依赖"，求解时要特殊对待（留待模块 D）。

## 8. 软件设计理念
- **组装小积木**：`Version` + `Constraint` + `Dependency` + DCF 解析，被组装成一个面向上层的 `PackageIndex`——自底向上、逐步求精的又一例。
- **定义清晰的接口**：`versions_of` / `len` 等构成"依赖图"的查询接口，为下一阶段（求解器）划清输入边界，让两个模块解耦。

## 9. 小测验（自测）
1. `rec.get("Package")?` 里的 `?` 在 `Option` 上做了什么？
2. 为什么索引值是 `Vec<Package>` 而不是单个 `Package`？
3. `entry(k).or_default().push(v)` 完成了什么常见操作？
4. 为什么 `versions_of` 返回 `&[Package]` 而不是 `Vec<Package>`？

## 10. 参考答案
1. 若 `get` 返回 `None`（无此字段），`?` 让 `from_record` 立即返回 `None`，于是这条不完整的记录被跳过。
2. 因为同一个包名可能存在多个版本；用 `Vec` 容纳所有版本，求解时再从中挑满足约束的。
3. "按键分组收集"：键不存在则插入一个默认空 `Vec`，然后把值追加进去。
4. 返回借用的切片**避免复制**整份数据；调用方只读，无需拥有所有权——更高效。

## 11. 下一步预告
模块 B（元数据）收官：我们已经能把 `PACKAGES` 变成可查询的依赖图。接下来开 PR 合入 `main`。
之后进入 **模块 D（依赖求解）**——给定要装的包 + 约束，从依赖图里解出一组自洽的版本。⚠️ 注意：引入求解器库 `pubgrub` 需要 `cargo` 联网拉取 crate；真正联网抓 `PACKAGES`（模块 C）、安装 R 包（模块 E）、对 pak 跑 benchmark（模块 G）会需要实时网络 / R / 系统安装——这些若在当前环境受限，会触发"资源墙"并交还给你。
