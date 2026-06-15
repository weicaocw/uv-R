# Step 38：并行下载——作用域线程 + 工作窃取

> 模块：K 并行 ｜ 对应提交：本步提交（commit 信息含 `Lesson: docs/lessons/step-38-parallel-download.md`）｜ 产物：`src/commands.rs`（`parallel_for_each` + `run_plan` 改为"并行预取 + 串行安装"）｜ 上一步：[Step 37](step-37-sync-cli.md)

## 0. 一句话目标
把"逐个下载 tarball"改成**并行下载**——这是 uv 的招牌吞吐优化。并且让这套并行编排**无需网络就能单测**。

## 1. 前置回顾
`install` / `sync` 都走 `run_plan`：逐个 `download` 再 `install_tarball`。下载是网络 IO，彼此**独立**（各写各的文件），天然适合并行。安装则要按序（先装依赖）、且 `R CMD INSTALL` 本身吃 CPU，先不并行。本步只并行**下载**。

## 2. "测试"先行（TDD 红）——关键设计：可注入
并行下载难测（要网络）。诀窍：把"对每个 item 做什么"做成**闭包参数** `f`，于是测试能传一个"只记录被调用"的假闭包，不碰网络：
```rust
// 每个 item 都被处理，无重无漏
parallel_for_each([a,b,c,d], 3, |it| { seen.insert(it.name) }) -> seen == {a,b,c,d}
// 并发度=1 也全做完
parallel_for_each([a,b], 1, |_| count+=1) -> count == 2
// 任一失败 → 整体 Err
parallel_for_each([ok,bad], 4, |it| if bad {Err}) -> Err 含 "boom"
// 空计划 → Ok
parallel_for_each([], 4, ...) -> Ok
```

## 3. 实现到通过（TDD 绿）
```rust
fn parallel_for_each<F>(plan: &[InstallItem], concurrency: usize, f: F) -> Result<(), String>
where F: Fn(&InstallItem) -> Result<(), String> + Sync {
    if plan.is_empty() { return Ok(()); }
    let next = std::sync::Mutex::new(0usize);        // 共享游标
    let errors = std::sync::Mutex::new(Vec::new());
    let workers = concurrency.clamp(1, plan.len());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let i = { let mut c = next.lock().unwrap(); let i = *c; *c += 1; i }; // 领下标
                if i >= plan.len() { break; }
                if let Err(e) = f(&plan[i]) { errors.lock().unwrap().push(e); }
            });
        }
    });
    let errs = errors.into_inner().unwrap();
    if errs.is_empty() { Ok(()) } else { Err(errs.join("; ")) }
}
```
`run_plan` 改为两阶段：**阶段一** `parallel_for_each(plan, jobs, |item| download(...))` 并行下载（`jobs = available_parallelism()`）；**阶段二**串行 `download`（缓存命中即返回）+ `install_tarball`。11 个 commands 测试全绿；真装 dotenv 回归通过。

## 4. 改了哪些文件 / 加了什么
- `src/commands.rs`：`tarball_path` 助手、`parallel_for_each`、`run_plan` 两阶段化；4 个并行测试。

## 5. 学到的语法 / 技巧（Rust 并发）
- **`std::thread::scope`（作用域线程）**：在作用域内 `spawn` 的线程，**保证在作用域结束前全部 join**。正因如此，线程闭包可以**借用**外部栈上的数据（`plan`、`next`、`errors`），编译器知道它们活得够久——普通 `thread::spawn` 要求 `'static`，做不到这种零拷贝借用。
- **`Mutex<T>` 共享可变状态**：多个线程要改同一个游标 / 错误表，用互斥锁保证一次只有一个线程在改。`lock().unwrap()` 拿到守卫，离开作用域自动解锁。**锁的持有时间要短**——这里领完下标立刻释放，下载在锁外进行。
- **工作窃取（work-stealing）**：不预先把任务**平均分**给线程（有的任务慢会拖累），而是让每个空闲线程**主动领下一个**。快的线程多领几个，慢的少领，自动负载均衡。一个共享游标 + 循环领取就实现了。
- **`F: Fn(...) + Sync` 约束**：闭包要被多个线程同时调用，必须 `Sync`（可安全跨线程共享引用）。捕获 `&Mutex` 的闭包天然满足。
- **`clamp(1, plan.len())`**：并发度兜进 `[1, 任务数]`——别开比任务还多的线程，也别开 0 个。
- **`into_inner()`**：作用域线程都结束后，没有别的持有者了，把 `Mutex` 拆开拿回里面的 `Vec`，无需再加锁。

## 6. 设计巧思 / 方法论
- **依赖注入换来可测性**：把副作用（真下载）作为闭包 `f` 注入，并行**编排**逻辑就与具体 IO 解耦，能用假闭包穷举单测（全做、并发度、报错、空集）。这是"把难测的东西推到参数上"的又一例。
- **并行下载、串行安装**：只并行能安全并行的部分（独立的下载），不动需要顺序 / 重 CPU 的部分（安装）。**识别哪些能并行**比"全部并行"更重要——错误的并行会引入数据竞争或破坏依赖顺序。
- **复用缓存做衔接**：阶段一并行下好，阶段二的 `download` 因 `dest.exists()` 直接返回。两阶段用"幂等的下载"无缝拼接，代码简单且不重复下载。

## 7. 领域知识（包管理 / 性能）
- **为什么并行下载是包管理器的核心优化**：装一组包时，瓶颈常是**网络往返**（每个 tarball 一次请求 + 传输）。串行下 N 个包要等 N 次往返；并行则约等于**最慢的那一个**。uv 正是靠激进并行 + 缓存把"装一堆包"做到飞快。
- **为什么安装先不并行**：`R CMD INSTALL` 要求依赖**先于**依赖者安装；并行安装得做依赖图调度（拓扑分层 + 层内并行），复杂且易错。先稳妥串行，把确定的吞吐收益（下载）拿到手。

## 8. 软件设计理念
- **正确性优先于速度**：并行只用在可证明安全处（独立下载）。宁可让安装慢一点、稳一点，也不为一点速度引入难调的并发 bug。
- **小而通用的并发原语**：`parallel_for_each` 不只能下载——任何"对一批东西并行做可失败的事"都能用它。造一个干净的通用件，胜过到处写裸线程。

## 9. 小测验（自测）
1. 为什么用 `thread::scope` 而不是 `thread::spawn`？前者让线程闭包能做到什么后者做不到的事？
2. "工作窃取"（共享游标领任务）相比"把任务平均分给每个线程"好在哪？
3. 为什么并行了**下载**却不并行**安装**？
4. 把 `f` 设计成闭包参数，对测试带来什么关键好处？

## 10. 参考答案
1. `thread::scope` 保证所有子线程在作用域结束前 join，因此线程闭包可以**借用**外部栈数据（`plan`、`Mutex`），无需 `'static` 或 `Arc` 克隆。`thread::spawn` 要求闭包 `'static`，得把数据 `move` 进去或用 `Arc` 包裹，更重。
2. 任务耗时不均时，平均分会让"分到慢任务"的线程拖后腿、其他线程闲着。共享游标让空闲线程主动多领，自动负载均衡，整体更快。
3. 安装有**依赖顺序**约束（依赖要先装好），且 `R CMD INSTALL` 吃 CPU；并行安装需要依赖图调度，复杂易错。下载彼此独立、是网络 IO，并行收益大且安全。
4. 可在**无网络**下单测并行编排：传一个记录调用 / 注入错误的假闭包，断言"每个都跑到、并发度生效、错误会传播、空集 OK"，把并行逻辑和真实 IO 解耦。

## 11. 下一步预告
Step 39：给并行加一个**旋钮**——`--jobs <N>` 控制并发度（默认 = CPU 核数），并发布 v0.9。
