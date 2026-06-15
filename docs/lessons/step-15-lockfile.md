# Step 15：lockfile（确定性序列化 + 往返）

> 模块：D 依赖求解 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-15-lockfile.md`）｜ 测试：✅ 20 passed ｜ 上一步：[Step 14](step-14-conflict-detection.md)

## 0. 一句话目标
把求解结果（包名 → 版本）写成**确定性的 lockfile 文本**，并能**读回来**（往返一致）。

## 1. 前置回顾
Step 14 能解出一组自洽的版本。要"可复现安装"，得把这组版本**锁定**到文件里——这就是 lockfile。本步实现它的写出与读回，并顺势给 `Version` 补上 `Display`（还记得早先讲 `{}` vs `{:?}` 时说"`Display` 要手写"吗？这次就来写）。

## 2. 先写测试（TDD·红）
```rust
#[test]
fn round_trips() {
    let res = resolution(); // {pkgA:1.2.0, pkgB:2.0.0}
    let text = render(&res);
    assert!(text.contains("pkgA 1.2.0"));
    let back = parse(&text).unwrap();
    assert_eq!(back, res);   // 写出再读回，内容一致
}
```
（`render`/`parse` 未实现时编译失败；红从略。）

## 3. 实现到通过（TDD·绿）
先给 `Version` 手写 `Display`（让 `{}` 能渲染 `1.2.0`）：
```rust
impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dotted = self.parts.iter().map(u64::to_string).collect::<Vec<_>>().join(".");
        write!(f, "{dotted}")
    }
}
```
再写 lockfile 的读写：
```rust
const HEADER: &str = "# uvr lockfile v1";

pub fn render(resolution: &BTreeMap<String, Version>) -> String {
    let mut out = String::from(HEADER);
    out.push('\n');
    for (name, version) in resolution {       // BTreeMap → 按名有序，确定性
        out.push_str(&format!("{name} {version}\n"));
    }
    out
}

pub fn parse(text: &str) -> Option<BTreeMap<String, Version>> {
    let mut map = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let (name, ver) = line.split_once(' ')?;   // 无空格 → 非法 → None
        map.insert(name.to_string(), Version::parse(ver)?);
    }
    Some(map)
}
```
`cargo test` → ✅ 20 passed（往返一致 + 拒绝畸形行）。

## 4. 改了哪些文件 / 加了什么
- 新增 `src/lockfile.rs`：`render` / `parse` + 2 个测试。
- `src/version.rs`：为 `Version` 手写 `impl Display`。
- `src/lib.rs`：新增 `pub mod lockfile;`。

## 5. 学到的语法 / 技巧
- **`impl Display`**：实现 `fmt`，用 `write!(f, ...)` 输出"给人看的"形态；有了它 `{}` 和 `.to_string()` 才可用。
- **`write!(f, "{dotted}")`**：往格式化器写字符串（内联捕获变量）。
- **`split_once(' ')`**：按第一个空格切成两半，返回 `Option<(&str, &str)>`。
- **迭代器 `map(u64::to_string).collect::<Vec<_>>().join(".")`**：把数字段拼成 `"1.2.0"`。
- **`?` on `Option`**：解析任一行失败即整体返回 `None`。
- **`BTreeMap` 的有序迭代**：保证输出确定性（同样的输入 → 同样的文件）。

## 6. 语言设计巧思
- **`Display` 必须手写、`Debug` 可派生**：呼应早先——"对用户友好的样子"是设计决定，编译器不猜；而 `Debug` 的机械展开可 `#[derive]`。
- **确定性输出**：用 `BTreeMap` 让 lockfile 按名排序，于是同样的解 → 逐字节相同的文件，**可 diff、可复现**。这正是 lockfile 的价值所在。

## 7. 领域知识
- **lockfile = 可复现安装的基石**：锁定每个包的确切版本，团队/CI 据此装出**完全一致**的环境。
- 类比：`Cargo.lock`（Rust）、`uv.lock`（Python uv）、`renv.lock`（R renv）都是同一思想。我们的格式最简，但概念一致；将来可换成 TOML/JSON（需 serde）。

## 8. 软件设计理念
- **序列化/反序列化对称**：`render` 与 `parse` 互逆；用**往返测试**（render→parse→相等）作为正确性的黄金标准。
- **最简够用格式优先**：纯文本、可读、可 diff、零依赖；等真有需要再上结构化格式。

## 9. 小测验（自测）
1. 为什么 `Display` 要手写，而 `Debug` 可以 `#[derive]`？
2. 用 `BTreeMap` 而非 `HashMap` 来渲染 lockfile，有什么好处？
3. `split_once(' ')` 返回什么？解析遇到没有空格的行会怎样？
4. 什么是"往返测试"？它验证了什么？

## 10. 参考答案
1. `Display` 是"给用户看的最终形态"，属于设计决定，编译器无法替你猜；`Debug` 只是机械地展开结构，故可自动派生。
2. `BTreeMap` 按键有序，渲染出的 lockfile 行顺序确定 → 同样的解产生逐字节相同的文件，便于版本控制 diff 与复现。
3. 返回 `Option<(&str, &str)>`：按第一个空格切两半。没有空格 → `None`，经 `?` 让 `parse` 返回 `None`（视为非法）。
4. "写出再读回再比较相等"。它验证序列化与反序列化互逆、没有信息在往返中丢失或损坏。

## 11. 下一步预告
模块 D（依赖求解）收官：从依赖图解出版本、检测冲突、锁成 lockfile，全离线、全测试覆盖。接着开 PR 合入 `main`。
之后做 **模块 F：最小命令行**（纯 std `std::env::args`，不引入 clap），把"读 PACKAGES → 求解 → 写 lockfile"串成一个能用的 `uvr` 命令——届时即可标记一个**可发布的 v0.1**（离线依赖求解器）。再往后的联网抓取 / 安装 / 对 pak 跑 benchmark 需要实时网络 / R / 系统安装，会触发资源墙。
