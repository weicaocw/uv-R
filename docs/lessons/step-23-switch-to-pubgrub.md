# Step 23：把 CLI 默认切到 pubgrub

> 模块：D′ pubgrub ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-23-switch-to-pubgrub.md`）｜ 测试：✅ 28 passed + 端到端照常 ｜ 上一步：[Step 22](step-22-pubgrub-adapter.md)

## 0. 一句话目标
让 `uvr lock` / `uvr install` 默认走 `resolve_pubgrub`——uvr 自此用**工业级回溯求解器**。

## 1. 前置回顾
Step 22 写好了 `resolve_pubgrub`，且**签名与手写 `resolve` 完全一致**（`Result<Map, ResolveError>`）。本步兑现那个伏笔：上层**无痛切换**实现。

## 2. "测试"：切换后既有测试与端到端流程必须全绿（回归保护）
没有新功能；切换的安全网是**所有既有测试 + 联网/安装端到端 demo 在切换后仍正常**。

## 3. 实现到通过（绿）
只改 `src/commands.rs` 三处：把 `use ...resolve` 改成 `resolve_pubgrub`，两处调用同样替换。
```rust
-use crate::resolver::{ResolveError, resolve};
+use crate::resolver::{ResolveError, resolve_pubgrub};
 ...
-        combined.extend(resolve(&index, root)?);
+        combined.extend(resolve_pubgrub(&index, root)?);
```
`cargo test` → ✅ 28 passed。端到端照常（现在由 pubgrub 驱动）：
```
$ uvr lock --repo https://jeroen.r-universe.dev jsonlite   → jsonlite 2.0.1
$ uvr install --repo https://gaborcsardi.r-universe.dev dotenv --lib ./r-lib  → installed dotenv 1.0.3.9000
```

## 4. 改了哪些文件 / 加了什么
- `src/commands.rs`：`lock_from_packages` 与 `install_plan` 改用 `resolve_pubgrub`（导入 + 两处调用）。

## 5. 学到的语法 / 技巧
- **签名一致 → 无痛替换**：因为 `resolve` 与 `resolve_pubgrub` 类型签名相同，换实现只动函数名，调用方逻辑零改动。
- **`use` 改名切换实现**：把导入从一个函数换成另一个，是最轻量的"切引擎"。

## 6. 语言设计巧思
- **面向接口编程的红利**：Step 22 特意让两个求解器**同型**，本步就收获了"换引擎不动管道"。这是"对接口编程、而非对实现编程"的具体好处。

## 7. 领域知识
- uvr 现在默认**回溯求解**：行为更贴近真实包管理器（uv / cargo 也用 PubGrub 系算法）。手写贪心版仍在库里，作为对照与教学。

## 8. 软件设计理念
- **渐进式替换（先并存、再切换）**：先让新实现与旧实现并存且被测试证明一致（Step 22），再切默认（Step 23）。风险小、可回退、可对比——这是替换核心组件的稳妥姿势。
- **回归保护**：用既有测试与端到端 demo 守住"换了引擎、行为不变（且更强）"。

## 9. 小测验（自测）
1. 为什么这次切换只改了三行、却没动任何调用方逻辑？
2. "先并存、再切换"相比"直接重写替换"有什么好处？
3. 切换后我们靠什么确认没把功能改坏？
4. 手写求解器现在还有用吗？

## 10. 参考答案
1. 因为 `resolve` 与 `resolve_pubgrub` 签名相同（同型可平替），上层按接口调用，换实现只需改函数名。
2. 可对比一致性、可随时回退、风险可控；直接重写替换则一旦出错难定位、难回退。
3. 既有的全部单元测试 + 联网求解 / 本地安装两个端到端 demo 在切换后仍全绿。
4. 有用：它是讲清求解原理的教材，也是 pubgrub 行为的对照基准（一致性测试）。

## 11. 下一步预告
模块 D′（pubgrub）收官：uvr 默认带回溯求解。接着开 PR 合入 `main`，打 **v0.3** 标签。
