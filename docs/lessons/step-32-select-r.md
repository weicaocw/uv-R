# Step 32：选哪个 R——按优先级解析

> 模块：M R 版本管理 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-32-select-r.md`）｜ 产物：`src/rversion.rs`（`version_matches` / `RSelectError` / `select_r` / `resolve_r`）｜ 上一步：[Step 31](step-31-pin-r-version.md)

## 0. 一句话目标
把"pin 的版本规格"和"发现到的安装清单"合起来，**按优先级选出**最终该用的那一个 R。

## 1. 前置回顾
- Step 30：`discover()` → 机器上有哪些 R。
- Step 31：`read_pin(dir)` → 这个项目钉了哪个版本（可能没钉）。

本步是把两者**合流**的决策：到底跑哪个 R？

## 2. "测试"先行（TDD 红）
先钉死匹配规则与选择规则：
```rust
version_matches("4.5",   4.5.2) == true    // 前缀匹配：4.5 命中 4.5.x
version_matches("4.6",   4.5.2) == false
version_matches("4.5.2.1", 4.5.2) == false // 规格比实际更具体 → 不命中

select_r(None, [])                       == Err(NoRInstalled)
select_r(None, [4.4.0, 4.5.2])           -> 4.5.2            // 无 pin 取最高
select_r(Some("4.4"), [4.4.0,4.4.2,4.5.2]) -> 4.4.2          // pin 内取最高
select_r(Some("3.6"), [4.5.2])           == Err(PinnedNotFound("3.6"))
```

## 3. 实现到通过（TDD 绿）
```rust
pub fn version_matches(spec: &str, v: &Version) -> bool {
    let full = v.to_string();
    let want: Vec<&str> = spec.split('.').collect();
    let have: Vec<&str> = full.split('.').collect();
    want.len() <= have.len() && want.iter().zip(&have).all(|(a, b)| a == b)
}

pub enum RSelectError { NoRInstalled, PinnedNotFound(String) }

pub fn select_r(pin: Option<&str>, installs: &[RInstall]) -> Result<RInstall, RSelectError> {
    if installs.is_empty() { return Err(RSelectError::NoRInstalled); }
    match pin {
        Some(spec) => installs.iter()
            .filter(|i| version_matches(spec, &i.version))
            .max_by(|a, b| a.version.cmp(&b.version)).cloned()
            .ok_or_else(|| RSelectError::PinnedNotFound(spec.to_string())),
        None => Ok(installs.iter()
            .max_by(|a, b| a.version.cmp(&b.version)).cloned().expect("已确认非空")),
    }
}
```
外加一层真·包装 `resolve_r(dir)` = `read_pin` + `discover` + `select_r`，给 CLI 用。18 个测试（17 + 1 ignored）全绿。

## 4. 改了哪些文件 / 加了什么
- `src/rversion.rs`：`version_matches`、`RSelectError`、`select_r`、`resolve_r` + 5 个测试。

## 5. 学到的语法 / 技巧
- **`Result<T, E>` 带原因的失败**：不像 `Option` 只说"没有"，`RSelectError` 用枚举区分两种失败（一个都没装 vs 钉了没装），CLI 能据此给不同的提示。
- **`max_by(|a,b| a.version.cmp(&b.version))`**：用我们 Step 01–07 给 `Version` 实现的 `Ord` 来挑最大值——又一次回报。返回 `Option`（空集合时 `None`）。
- **`.cloned()`**：迭代器产出的是 `&RInstall`（借用），`.cloned()` 把选中的那个**复制**成拥有的 `RInstall` 返回，避免把生命周期借用泄漏到函数外。
- **`ok_or_else(err_fn)`**：`Option` → `Result`。`None` 时调用闭包造一个错误。用 `_else`（惰性）而非 `ok_or(...)`，避免每次都构造那个 `String`（clippy 的 `or_fun_call` 提醒）。
- **`zip` + `all` 做前缀匹配**：`want.iter().zip(&have).all(|(a,b)| a==b)` 配合 `want.len() <= have.len()`，简洁表达"want 是 have 的前缀"。
- **`pin.as_deref()`**：`Option<String>` → `Option<&str>`，把"拥有的字符串"借成"字符串切片"传给 `select_r`，零拷贝。

## 6. 设计巧思 / 方法论
- **优先级即策略**：pin > 最高版本。这条规则只活在 `select_r` 一处——纯函数、好测、好改。将来要加"命令行 `--r` 显式指定"或"环境变量覆盖"，只需在更上层（`resolve_r` / CLI）插一层，`select_r` 不动。
- **前缀匹配的弹性**：pin 写 `4.5` 表示"任意 4.5.x 都行"，写 `4.5.2` 表示"就要这个补丁版"。一套规则覆盖宽松到严格，把"多严"的选择权交给用户。

## 7. 领域知识（R / 工具链）
- **R 的版本兼容性**：R 的主次版本（4.5）内补丁版（4.5.0 → 4.5.2）通常兼容；跨次版本（4.4 → 4.5）则可能需要重装包。所以 pin 到 `4.5` 往往就够，既锁住兼容性又不必每个补丁都改文件。
- **对标 uv 的解析优先级**：uv 选 Python 的优先级大致是 命令行 > 环境变量 > `.python-version` > 系统发现。uvr 现在实现了"pin > 发现最高"这条主线，更高优先级的覆盖留作后续。

## 8. 软件设计理念
- **决策与副作用分离**：`select_r` 是**纯决策**（给数据、出结论，可彻底单测）；`resolve_r` 才去碰真实文件系统和进程。把"难测的 IO"压到最薄的一层，"核心策略"留在能被穷举测试的纯函数里——这是本模块（也是整个 uvr）反复用的结构。

## 9. 小测验（自测）
1. `select_r` 为什么返回 `Result` 而不是 `Option`？区分两种错误带来什么好处？
2. pin `4.4` 在 `[4.4.0, 4.4.2, 4.5.2]` 里为什么选 4.4.2 而不是 4.5.2？
3. `version_matches("4.5.2.1", 4.5.2)` 为什么是 false？这条规则防住了什么误用？
4. 将来要加"命令行 `--r 4.4` 覆盖 pin"，应该改哪个函数？为什么不动 `select_r`？

## 10. 参考答案
1. 因为失败有**不同含义**：一个 R 都没装（让用户去装）vs 钉的版本没装（让用户改 pin 或装那个版本）。`Result` + 枚举能把原因带给 CLI，给出对症的提示；`Option` 只能说"没选到"。
2. pin 先**过滤**出匹配 `4.4` 的集合 `{4.4.0, 4.4.2}`，再在其中取最高 → 4.4.2。4.5.2 不匹配 `4.4`，被排除。
3. 规格 `4.5.2.1` 有 4 段，比实际版本 `4.5.2`（3 段）更长，`want.len() <= have.len()` 不成立 → false。防住"钉一个比任何已装 R 都更具体的版本却以为命中"的误用。
4. 在更上层（`resolve_r` 或 CLI 解析）：若有 `--r`，就把它当 pin 传给 `select_r`（或直接定位）。`select_r` 只关心"给定 pin + 安装清单怎么选"，覆盖优先级是**上层**的事——保持它单一职责、不被新需求污染。

## 11. 下一步预告
Step 33：把这一切接到命令行——`uvr r list` / `uvr r which` / `uvr r pin <版本>`，让你在终端里真正用上 R 版本管理。
