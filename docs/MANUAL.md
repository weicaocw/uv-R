# uvr 用户手册 / User Manual

> 适用版本 / Applies to: **v0.13**　·　中英文对照 / Bilingual (中文 → English)
> 教程式逐步讲解见 `docs/lessons/`；本手册是**面向使用**的参考。
> For a step-by-step tutorial see `docs/lessons/`; this manual is a **usage-oriented** reference.

## 目录 / Table of contents
1. [简介 / Introduction](#1-简介--introduction)
2. [安装与构建 / Install & build](#2-安装与构建--install--build)
3. [快速上手 / Quickstart](#3-快速上手--quickstart)
4. [命令参考 / Command reference](#4-命令参考--command-reference)
5. [R 版本管理 / R version management](#5-r-版本管理--r-version-management)
6. [缓存模型 / Caching model](#6-缓存模型--caching-model)
7. [项目布局 / Project layout](#7-项目布局--project-layout)
8. [与 pak 对比 / vs pak](#8-与-pak-对比--vs-pak)
9. [排错与 FAQ / Troubleshooting & FAQ](#9-排错与-faq--troubleshooting--faq)
10. [设计与边界 / Design & limits](#10-设计与边界--design--limits)

---

## 1. 简介 / Introduction

**中文**　`uvr` 是一个用 Rust 写的、类似 [uv](https://github.com/astral-sh/uv) / Cargo 的 **R 语言包管理器**。它做三件事：从一个或多个 R 仓库**求解依赖**并写 lockfile、把包**下载并安装**进项目本地库、以及**管理本机的 R 版本**（发现 / 钉版本 / 选择）。设计目标是快、确定、可复现，且**绝不污染**你的全局 / 用户级 R 环境。

**English**　`uvr` is a uv/Cargo-style **package manager for R**, written in Rust. It does three things: **resolve dependencies** from one or more R repositories and write a lockfile; **download and install** packages into a project-local library; and **manage the R interpreter version** on your machine (discover / pin / select). It aims to be fast, deterministic, and reproducible, and to **never pollute** your global/user R environment.

---

## 2. 安装与构建 / Install & build

**中文**　目前从源码构建（需要 Rust 工具链；安装见 <https://rustup.rs>）：

**English**　Build from source for now (needs the Rust toolchain; see <https://rustup.rs>):

```sh
git clone https://github.com/weicaocw/uv-R.git
cd uv-R
cargo build --release        # 产物 / binary: target/release/uvr
./target/release/uvr         # 打印用法 / prints usage
```

**中文**　把 `target/release/uvr` 加进 `PATH` 即可全局调用。开发时也可用 `cargo run -- <子命令> ...`。

**English**　Put `target/release/uvr` on your `PATH` to call it anywhere. During development you can also use `cargo run -- <subcommand> ...`.

---

## 3. 快速上手 / Quickstart

```sh
# 1) 求解依赖并打印 lockfile（从真实仓库联网）/ resolve & print a lockfile (live)
uvr lock --repo https://jeroen.r-universe.dev jsonlite

# 2) 安装进项目本地库（默认 ./r-lib）/ install into a project-local lib (default ./r-lib)
uvr install --repo https://gaborcsardi.r-universe.dev dotenv

# 3) 看本机有哪些 R、用哪个 / see which R's exist and which is selected
uvr r list
uvr r which

# 4) 把项目钉到某个 R 版本 / pin the project to an R version
uvr r pin 4.5
```

**中文**　就这些。下面是每条命令的细节。

**English**　That's it. Details for each command follow.

---

## 4. 命令参考 / Command reference

### `uvr lock` — 求解依赖、输出 lockfile / resolve and emit a lockfile

```
uvr lock <PACKAGES-file> <package>...
uvr lock --repo <url> [--repo <url2> ...] <package>...
```

**中文**
- 形式一：从**本地** `PACKAGES` 文件离线求解。
- 形式二：从**一个或多个**真实仓库抓取 `PACKAGES`（走暖缓存）、合并后求解。
- 用工业级 **pubgrub** 回溯求解器；跳过随 R 自带的 base/recommended 包。
- 结果（lockfile）打印到标准输出，形如：

**English**
- Form 1: resolve offline from a **local** `PACKAGES` file.
- Form 2: fetch `PACKAGES` from **one or more** real repositories (warm-cached), merge, and resolve.
- Uses the industrial **pubgrub** backtracking resolver; skips R's bundled base/recommended packages.
- The lockfile is printed to stdout, like:

```
# uvr lockfile v1
jsonlite 2.0.1
```

**中文**　多个 `--repo` 会被**合并**为一张索引，于是能解开"A 仓库的包依赖 B 仓库的包"这类跨仓库依赖。
**English**　Multiple `--repo` flags are **merged** into one index, so cross-repository dependencies (a package in repo A depending on one in repo B) resolve correctly.

### `uvr install` — 下载并安装到项目本地库 / download & install into a project-local lib

```
uvr install --repo <url> [--repo <url2> ...] [--lib <dir>] [--jobs <N>] <package>...
```

**中文**
- 求解（同 `lock`）→ 按**每个包自己的仓库**下载源码 tarball → 用选中的 R 跑 `R CMD INSTALL -l <lib>`。
- `--lib <dir>`：安装目标目录，默认 `./r-lib`。`-l` 把安装**限定在该目录**，因此**不碰**全局 / 用户级 R 库。
- `--jobs <N>`：并行下载的并发度，默认 = CPU 核数。tarball 并行预取、安装串行（见 [第 6 节](#6-缓存模型--caching-model)）。
- 安装前会打印用的是哪个 R，例如 `→ 使用 R / using R 4.5.2: /usr/local/bin/R`（见 [第 5 节](#5-r-版本管理--r-version-management)）。

**English**
- Resolve (like `lock`) → download source tarballs from **each package's own repo** → run `R CMD INSTALL -l <lib>` with the selected R.
- `--lib <dir>`: install target, default `./r-lib`. `-l` **confines** the install to that directory, so it **never touches** the global/user R library.
- `--jobs <N>`: concurrency for parallel downloads, default = CPU count. Tarballs are prefetched in parallel; installs run serially (see [§6](#6-缓存模型--caching-model)).
- It first prints which R it uses, e.g. `→ 使用 R / using R 4.5.2: /usr/local/bin/R` (see [§5](#5-r-版本管理--r-version-management)).

**中文**　在 R 里使用这个本地库：`.libPaths("./r-lib"); library(dotenv)`，或设环境变量 `R_LIBS_USER=./r-lib`。
**English**　To use the local library from R: `.libPaths("./r-lib"); library(dotenv)`, or set `R_LIBS_USER=./r-lib`.

### `uvr sync` — 按 lockfile 还原环境 / restore from a lockfile

```
uvr sync [--repo <url> ...] [--lib <dir>] [--jobs <N>] [<lockfile>]
```

**中文**
- 读 lockfile（省略则默认 `uvr.lock`），**不求解**，严格安装其中锁定的版本——可复现、防漂移。
- **完整性校验**：v3 锁文件记了校验和（SHA256 或 CRAN 的 MD5）；下载后、安装前校验，**不符即报错拒装**（防损坏 / 篡改）。仓库没给校验和时跳过。
- `--repo` **可选**：v2/v3 锁文件自带来源仓库，直接 `uvr sync` 即可；仅旧的 v1 锁文件（无来源）才需要 `--repo` 兜底。`--lib`：目标库，默认 `./r-lib`。
- 锁定的版本在来源里找不到（被下架 / 换了仓库），或 v1 锁文件没给 `--repo`，会报错并指名是哪个包。
- 典型工作流：`uvr lock --repo ... pkg > uvr.lock`（提交进库）→ 队友 / CI / 新机器 **`uvr sync`**（无需 `--repo`）还原一模一样的依赖。

**English**
- Read a lockfile (default `uvr.lock` if omitted), **without resolving**, and install exactly the locked versions — reproducible, no drift.
- **Integrity check**: a v3 lockfile records a checksum (SHA256 or CRAN's MD5); it is verified after download and before install, **erroring out on a mismatch** (guards corruption/tampering). Skipped when the repo provides no checksum.
- `--repo` is **optional**: a v2/v3 lockfile carries its source repos, so a bare `uvr sync` works; only old v1 lockfiles (no sources) need `--repo`. `--lib`: target lib, default `./r-lib`.
- If a locked version isn't found in the sources (yanked / repo changed), or a v1 lockfile lacks `--repo`, it errors and names the package.
- Workflow: `uvr lock --repo ... pkg > uvr.lock` (commit it) → teammates / CI / a fresh machine run **`uvr sync`** (no `--repo`) to restore identical deps.

### `uvr r` — 管理 R 版本 / manage R versions

```
uvr r list                 # 列出发现到的所有 R（* 标当前选中）/ list discovered R's (* = selected)
uvr r which                # 打印当前项目会用的 R / print the R this project will use
uvr r pin [<version>]      # 钉版本到 ./.R-version；省略则钉当前解析值 / pin to ./.R-version
uvr r install <version>    # 不自己装：委托 rig 或给出指引 / hands off to rig or prints guidance
```

详见 [第 5 节](#5-r-版本管理--r-version-management)。/ See [§5](#5-r-版本管理--r-version-management).

---

## 5. R 版本管理 / R version management

**中文**　这是对标 uv `uv python` 家族的能力。uvr **不自己编译 / 安装 R**（就像 uv 不自己编译 Python），只负责**发现、选择、使用**已装的 R，并把"获取新 R"交给 `rig`。

**English**　This mirrors uv's `uv python` family. uvr **does not compile/install R itself** (just as uv doesn't build Python); it **discovers, selects, and uses** already-installed R's, and delegates **acquiring a new R** to `rig`.

### 发现 / Discovery
**中文**　`uvr r list` 扫描 `PATH` 中每个目录下的 `R`，外加一组已知安装位置（Homebrew、CRAN framework、Linux 路径），逐个跑 `R --version`，按版本去重。
**English**　`uvr r list` scans `R` under each `PATH` dir plus a set of known locations (Homebrew, the CRAN framework, Linux paths), runs `R --version` on each, and dedups by version.

### 钉版本 / Pinning（`.R-version`）
**中文**　`uvr r pin 4.5` 在当前目录写一个 `.R-version` 文件（对标 `.python-version`）。把它**提交进版本库**，团队就共享同一个 R 版本。pin 可以是部分版本：`4.5` 表示"任意 4.5.x"，`4.5.2` 表示精确补丁版。
**English**　`uvr r pin 4.5` writes a `.R-version` file in the current directory (analogue of `.python-version`). **Commit it** and your team shares one R version. A pin may be partial: `4.5` means "any 4.5.x", `4.5.2` means that exact patch.

### 选择优先级 / Selection precedence
**中文**
1. **`.R-version` 钉的版本**（前缀匹配；匹配多个时取其中最高）。
2. 否则取发现到的**最高版本**。
3. 一个 R 都没发现 → 报错 `no R installation found`；钉了却没装 → 报错 `pinned R version X is not installed`。

**English**
1. The **`.R-version` pin** (prefix match; if several match, the highest among them).
2. Otherwise the **highest** discovered version.
3. No R at all → `no R installation found`; pinned-but-absent → `pinned R version X is not installed`.

### 安装由 install 使用 / Used by install
**中文**　`uvr install` 用上面解析出的那个 R 跑 `R CMD INSTALL`。若你 pin 了一个**没装**的版本，`install` 会**报错中止**，绝不偷偷换一个 R——保证可复现。
**English**　`uvr install` runs `R CMD INSTALL` with the R resolved above. If you pinned a version that isn't installed, `install` **aborts with an error** rather than silently swapping R — preserving reproducibility.

### 获取新 R / Acquiring a new R
**中文**　`uvr r install 4.4` **不**直接安装（那是系统级操作、需权限、要下载几百 MB）。检测到 `rig` 就提示 `rig add 4.4`；否则指向 rig 与 CRAN。装好后 `uvr r list` 会自动发现它。
**English**　`uvr r install 4.4` does **not** install directly (a system-level action needing privileges and a large download). If `rig` is present it suggests `rig add 4.4`; otherwise it points to rig and CRAN. Once installed, `uvr r list` discovers it automatically.

---

## 6. 缓存模型 / Caching model

**中文**　uvr 把可复用的东西缓存到项目下的 `.uvr-cache/`：
- `.uvr-cache/meta/`：各仓库抓来的 `PACKAGES`（元数据）。
- `.uvr-cache/tarballs/`：下载过的源码 tarball。

重复 `lock` / `install` 命中暖缓存、**免联网**，实测元数据 `lock --repo` 冷 ~640ms → 暖 ~7ms（~94×）。删 `.uvr-cache/` 即清缓存（下次重新抓）。建议把它加入 `.gitignore`。

`install` / `sync` 会**并行**下载所有 tarball（`--jobs <N>`，默认 = CPU 核数）再安装；下载彼此独立故可安全并行，`R CMD INSTALL` 按**依赖顺序（拓扑序）**串行进行——被依赖的包先装。

**English**　uvr caches reusable artifacts under `.uvr-cache/` in your project:
- `.uvr-cache/meta/`: fetched `PACKAGES` (metadata) per repo.
- `.uvr-cache/tarballs/`: downloaded source tarballs.

Repeated `lock` / `install` hit the warm cache, **no network needed** — measured `lock --repo` cold ~640ms → warm ~7ms (~94×). Delete `.uvr-cache/` to clear it (re-fetched next time). Add it to `.gitignore`.

`install` / `sync` download all tarballs **in parallel** (`--jobs <N>`, default = CPU count) then install; downloads are independent so parallelism is safe, while `R CMD INSTALL` runs serially in **dependency (topological) order** — dependencies first.

---

## 7. 项目布局 / Project layout

```
your-project/
├── .R-version          # 钉的 R 版本（建议提交）/ pinned R version (commit it)
├── r-lib/              # 项目本地 R 库（建议 gitignore）/ project-local R lib (gitignore)
├── .uvr-cache/         # 元数据 + tarball 缓存（建议 gitignore）/ caches (gitignore)
└── uvr.lock            # 你保存的 lockfile（可选，由 `lock` 重定向）/ your saved lockfile (optional)
```

**中文**　`uvr lock ... > uvr.lock` 可把 lockfile 存盘并提交，作为可复现的依赖快照。
**English**　`uvr lock ... > uvr.lock` saves the lockfile to commit as a reproducible dependency snapshot.

`.gitignore` 建议 / suggested:
```
/r-lib
/.uvr-cache
```

---

## 8. 与 pak 对比 / vs pak

**中文**　完整、可复现、多轴的对照见 [`BENCHMARK.md`](../BENCHMARK.md)。摘要（本机，纯 R 包 dotenv）：一次性**解析** uvr ~6ms vs pak ~5415ms；**暖缓存 lock** ~6ms；**端到端安装** uvr 暖 ~1575ms vs pak 暖 ~5478ms。诚实口径：uvr 赢在结构（原生、无 R 启动、暖缓存、pubgrub）；`R CMD INSTALL` 的 ~1.5s 是两边共担的地板；含编译的大包待 binary 支持后再公平比。

**English**　See [`BENCHMARK.md`](../BENCHMARK.md) for a full, reproducible, multi-axis comparison. Summary (this machine, pure-R `dotenv`): one-shot **resolve** uvr ~6ms vs pak ~5415ms; **warm-cache lock** ~6ms; **end-to-end install** uvr warm ~1575ms vs pak warm ~5478ms. Honest scope: uvr wins structurally (native, no R startup, warm cache, pubgrub); the ~1.5s of `R CMD INSTALL` is a shared floor; compiled packages await binary support before a fair comparison.

---

## 9. 排错与 FAQ / Troubleshooting & FAQ

**中文**
- **`no R installation found`**：`PATH` 上和已知位置都没找到 R。装一个（见 `uvr r install`）或把 R 加进 `PATH`。
- **`pinned R version X is not installed`**：`.R-version` 钉的版本没装。改 pin（`uvr r pin`）或装那个版本（`rig add X`）。
- **`install` 报 `R CMD INSTALL` 退出码非零**：通常是该包需要**编译**且缺系统依赖（编译器 / 系统库）。装齐工具链，或改用提供 binary 的环境（见下）。
- **联网失败 / fetch failed**：检查网络与仓库 URL；元数据会缓存，断网时旧缓存仍可用。
- **想清缓存**：删 `.uvr-cache/`。
- **想换库目录**：`--lib <dir>`。

**English**
- **`no R installation found`**: no R on `PATH` or known locations. Install one (see `uvr r install`) or add R to `PATH`.
- **`pinned R version X is not installed`**: the `.R-version` pin isn't installed. Change the pin (`uvr r pin`) or install it (`rig add X`).
- **`install` reports a non-zero `R CMD INSTALL` exit**: usually the package needs **compilation** and is missing system deps (a compiler / system libraries). Install the toolchain, or use an environment with binaries (below).
- **fetch failed**: check the network and repo URL; metadata is cached, so a stale cache still works offline.
- **Clear the cache**: delete `.uvr-cache/`.
- **Change the lib dir**: `--lib <dir>`.

---

## 10. 设计与边界 / Design & limits

**中文**
- **绝不污染全局环境**：安装一律走 `R CMD INSTALL -l <项目库>`；R 版本只发现 / 选择、获取交给 rig。这是项目的硬底线。
- **当前装的是源码包**：纯 R 包零差别；含编译的包目前走源码编译（pak 用预编译 binary 更快）。预编译 binary（模块 J）在本开发环境受限（此 macOS 的 R 默认 `pkgType=source`），留作后续。
- **并行下载 / 安装（模块 K）**：尚未实现，属吞吐优化，未来加入。
- **多仓库刷新策略**：暂无 TTL/etag，删缓存即重抓。

**English**
- **Never pollute the global env**: installs always go through `R CMD INSTALL -l <project lib>`; R versions are only discovered/selected, with acquisition delegated to rig. This is a hard line.
- **Source packages for now**: pure-R packages are identical; compiled packages currently build from source (pak is faster via prebuilt binaries). Prebuilt binaries (Module J) are constrained in this dev environment (this macOS R defaults to `pkgType=source`) and are deferred.
- **Parallel download/install (Module K)**: not yet implemented; a throughput optimization for the future.
- **Multi-repo refresh**: no TTL/etag yet; delete the cache to re-fetch.

---

> 反馈与贡献 / Feedback & contributions: <https://github.com/weicaocw/uv-R>
