//! 命令实现：把"读元数据 → 求解 → 渲染 lockfile / 下载安装"串起来，供命令行入口调用。
//!
//! 纯逻辑（求解、安装计划）做成可测函数；真正的网络 / 文件 / 进程 IO 在
//! `install_packages` 与 `main` 里。支持**多仓库来源**。

use crate::install;
use crate::lockfile;
use crate::lockfile::Locked;
use crate::metadata::PackageIndex;
use crate::resolver::{ResolveError, resolve_pubgrub};
use crate::version::Version;
use std::collections::BTreeMap;
use std::path::Path;

/// tarball 在 `download_dir` 里的落点路径。
fn tarball_path(download_dir: &Path, item: &InstallItem) -> std::path::PathBuf {
    download_dir.join(format!("{}_{}.tar.gz", item.name, item.version))
}

/// 对 `plan` 里每个 item 并行执行闭包 `f`，最多 `concurrency` 个线程同时跑。
///
/// 用**作用域线程**（`thread::scope`）+ 一个共享游标（`Mutex<usize>`）做"工作窃取"：
/// 每个线程循环领取下一个待办下标，干完再领，直到取完。任一 `f` 失败则收集错误、整体返回 `Err`。
/// 把 `f` 作为参数注入，使这套并行编排**无需网络即可单测**（测试传一个记录调用的闭包）。
fn parallel_for_each<F>(plan: &[InstallItem], concurrency: usize, f: F) -> Result<(), String>
where
    F: Fn(&InstallItem) -> Result<(), String> + Sync,
{
    if plan.is_empty() {
        return Ok(());
    }
    let next = std::sync::Mutex::new(0usize);
    let errors = std::sync::Mutex::new(Vec::<String>::new());
    let workers = concurrency.clamp(1, plan.len());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    // 领取下一个下标（锁只在这一瞬间持有）。
                    let i = {
                        let mut cursor = next.lock().unwrap();
                        let i = *cursor;
                        *cursor += 1;
                        i
                    };
                    if i >= plan.len() {
                        break;
                    }
                    if let Err(e) = f(&plan[i]) {
                        errors.lock().unwrap().push(e);
                    }
                }
            });
        }
    });
    let errs = errors.into_inner().unwrap();
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs.join("; "))
    }
}

/// 按计划安装到 `lib_dir`：**先并行预取所有 tarball**，再按顺序 `R CMD INSTALL`。
///
/// `install`（求解后）与 `sync`（按 lockfile）共用。下载彼此独立、可并行（uv 的招牌吞吐优化）；
/// 安装仍串行（`download` 命中已下好的缓存即刻返回）。`jobs` 控制并行下载的并发度。
fn run_plan(
    plan: &[InstallItem],
    lib_dir: &Path,
    download_dir: &Path,
    r_bin: &Path,
    jobs: usize,
) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(download_dir).map_err(|e| e.to_string())?;
    // 阶段一：并行下载所有 tarball。
    parallel_for_each(plan, jobs, |item| {
        install::download(&item.url, &tarball_path(download_dir, item))
    })?;
    // 阶段二：串行安装（download 此时多为缓存命中）。
    let mut installed = Vec::new();
    for item in plan {
        let tarball = tarball_path(download_dir, item);
        install::download(&item.url, &tarball)?;
        install::install_tarball(&tarball, lib_dir, r_bin)?;
        installed.push(format!("{} {}", item.name, item.version));
    }
    Ok(installed)
}

/// 一个元数据来源：(PACKAGES 文本, 仓库基址)。
pub type Source = (String, String);

/// 把多个来源合并成一个索引（每个包记住自己的仓库）。
fn build_index(sources: &[Source]) -> PackageIndex {
    let mut index = PackageIndex::default();
    for (text, repo) in sources {
        index.merge(PackageIndex::from_repo(text, repo));
    }
    index
}

/// 对合并后的索引求解所有根包，得到"包名 → 版本"。
fn resolve_all(
    index: &PackageIndex,
    roots: &[String],
) -> Result<BTreeMap<String, Version>, ResolveError> {
    let mut combined = BTreeMap::new();
    for root in roots {
        combined.extend(resolve_pubgrub(index, root)?);
    }
    Ok(combined)
}

/// 给求解结果（name → version）补上每个包的**来源仓库**，得到可写进 v2 lockfile 的锁定项。
fn with_repos(
    index: &PackageIndex,
    resolved: BTreeMap<String, Version>,
) -> BTreeMap<String, Locked> {
    resolved
        .into_iter()
        .map(|(name, version)| {
            let found = index
                .versions_of(&name)
                .iter()
                .find(|p| p.version == version);
            let repo = found.map(|p| p.repo.clone()).unwrap_or_default();
            let hash = found.map(|p| p.hash.clone()).unwrap_or_default();
            (
                name,
                Locked {
                    version,
                    repo,
                    hash,
                },
            )
        })
        .collect()
}

/// 多源求解并渲染 lockfile 文本（v3：含来源仓库与校验和）。
pub fn lock_from_sources(sources: &[Source], roots: &[String]) -> Result<String, ResolveError> {
    let index = build_index(sources);
    let resolved = resolve_all(&index, roots)?;
    Ok(lockfile::render(&with_repos(&index, resolved)))
}

/// 单一本地 `PACKAGES` 文本的便捷封装（来源仓库为空）。
pub fn lock_from_packages(packages_text: &str, roots: &[String]) -> Result<String, ResolveError> {
    lock_from_sources(&[(packages_text.to_string(), String::new())], roots)
}

/// 一项待安装：包名、版本、源码 tarball 的 URL（指向该包自己的仓库）+ 校验和（可空）。
#[derive(Debug, PartialEq, Eq)]
pub struct InstallItem {
    pub name: String,
    pub version: String,
    pub url: String,
    /// 期望校验和（带算法前缀，如 `sha256:…`）；空则不校验。
    pub hash: String,
}

/// 纯函数：把待装项按**依赖顺序**重排（依赖在前、依赖者在后）——拓扑排序。
///
/// 边取自 `index` 里每个包的依赖字段，只保留指向**本批待装集合内**的依赖（跳过 base/recommended
/// 与不在集合里的）。用 Kahn 算法：每轮在"入度为 0（依赖都装好了）"里取**字典序最小**的，
/// 保证确定性输出。万一成环（求解后理论上不会），剩余项按名字追加，绝不丢包。
/// `index` 信息不足（如自包含 sync 没传 `--repo`）时，没有边 → 退化为按名字排序。
fn topo_order(items: Vec<InstallItem>, index: &PackageIndex) -> Vec<InstallItem> {
    use std::collections::{BTreeMap, BTreeSet};

    let names: BTreeSet<String> = items.iter().map(|i| i.name.clone()).collect();

    // 每个包"在集合内、需先装好"的依赖集合。
    let mut deps_of: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for it in &items {
        let mut ds = BTreeSet::new();
        if let Some(pkg) = index
            .versions_of(&it.name)
            .iter()
            .find(|p| p.version.to_string() == it.version)
        {
            for d in &pkg.depends {
                if d.name != it.name && names.contains(&d.name) {
                    ds.insert(d.name.clone());
                }
            }
        }
        deps_of.insert(it.name.clone(), ds);
    }

    // 入度 = 还有多少依赖没装；dependents：某依赖被谁依赖（用于装好后递减入度）。
    let mut indeg: BTreeMap<String, usize> = deps_of
        .iter()
        .map(|(n, ds)| (n.clone(), ds.len()))
        .collect();
    let mut dependents: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (n, ds) in &deps_of {
        for d in ds {
            dependents.entry(d.clone()).or_default().push(n.clone());
        }
    }

    // 就绪集合（入度 0），每次取名字最小者 → 确定性。
    let mut ready: BTreeSet<String> = indeg
        .iter()
        .filter(|(_, v)| **v == 0)
        .map(|(n, _)| n.clone())
        .collect();
    let mut order: Vec<String> = Vec::new();
    while let Some(n) = ready.iter().next().cloned() {
        ready.remove(&n);
        order.push(n.clone());
        if let Some(deps) = dependents.get(&n) {
            for dep in deps {
                if let Some(e) = indeg.get_mut(dep) {
                    *e -= 1;
                    if *e == 0 {
                        ready.insert(dep.clone());
                    }
                }
            }
        }
    }

    // 成环兜底：把剩余项按名字补上，绝不丢包。
    if order.len() < items.len() {
        let placed: BTreeSet<String> = order.iter().cloned().collect();
        for it in &items {
            if !placed.contains(&it.name) {
                order.push(it.name.clone());
            }
        }
    }

    // 名字 → 项，按拓扑序取回。
    let mut by_name: BTreeMap<String, InstallItem> =
        items.into_iter().map(|i| (i.name.clone(), i)).collect();
    order
        .into_iter()
        .filter_map(|n| by_name.remove(&n))
        .collect()
}

/// 纯函数：从多个来源解析出"要装哪些包、各自 tarball URL"——按**每个包自己的仓库**拼地址。
pub fn install_plan(
    sources: &[Source],
    roots: &[String],
) -> Result<Vec<InstallItem>, ResolveError> {
    let index = build_index(sources);
    let resolved = resolve_all(&index, roots)?;
    let items: Vec<InstallItem> = resolved
        .into_iter()
        .map(|(name, version)| {
            // 找到该 name+version 的包，用它自己的仓库拼下载地址、取它的校验和
            let found = index
                .versions_of(&name)
                .iter()
                .find(|p| p.version == version);
            let repo = found.map(|p| p.repo.as_str()).unwrap_or("");
            let hash = found.map(|p| p.hash.clone()).unwrap_or_default();
            let version = version.to_string();
            let url = install::tarball_url(repo, &name, &version);
            InstallItem {
                name,
                version,
                url,
                hash,
            }
        })
        .collect();
    Ok(topo_order(items, &index)) // 依赖先于依赖者
}

/// 按计划下载并安装到**项目本地库** `lib_dir`；tarball 暂存到 `download_dir`。
///
/// `r_bin` 是用来跑 `R CMD INSTALL` 的 R 可执行文件（由 `rversion::resolve_r` 选出）。
pub fn install_packages(
    sources: &[Source],
    roots: &[String],
    lib_dir: &Path,
    download_dir: &Path,
    r_bin: &Path,
    jobs: usize,
) -> Result<Vec<String>, String> {
    let plan = install_plan(sources, roots).map_err(|e| format!("{e:?}"))?;
    run_plan(&plan, lib_dir, download_dir, r_bin, jobs)
}

/// 纯函数：按 lockfile（已锁定的 `name → 锁定项`）拼下载计划。
///
/// 与 `install_plan` 的关键区别：**不求解**。它严格安装 lockfile 里写死的版本，
/// 即使仓库里已有更高版本——这正是 `sync` 的意义：可复现，杜绝"求解漂移"。
///
/// 来源仓库**优先用 lockfile 里记的**（v2，自包含）；若为空（v1 旧锁文件）则回退到
/// `sources` 索引里查找。两者都拿不到则报错并指名是哪个包。
pub fn sync_plan(
    locked: &BTreeMap<String, Locked>,
    sources: &[Source],
) -> Result<Vec<InstallItem>, String> {
    let index = build_index(sources);
    let items = locked
        .iter()
        .map(|(name, lk)| {
            let repo = if lk.repo.is_empty() {
                // v1 回退：从提供的 --repo 索引里找该包该版本的仓库。
                index
                    .versions_of(name)
                    .iter()
                    .find(|p| p.version == lk.version)
                    .map(|p| p.repo.clone())
                    .ok_or_else(|| {
                        format!(
                            "锁定的 {name} {} 找不到来源（lockfile 无仓库且 --repo 里也没有）/ no source for locked {name} {}",
                            lk.version, lk.version
                        )
                    })?
            } else {
                lk.repo.clone()
            };
            let url = install::tarball_url(&repo, name, &lk.version.to_string());
            Ok(InstallItem {
                name: name.clone(),
                version: lk.version.to_string(),
                url,
                hash: lk.hash.clone(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    // 有索引（传了 --repo）时按依赖排序；自包含 sync 无图则退化为名字序。
    Ok(topo_order(items, &index))
}

/// 读 lockfile 文本，按其中锁定的版本下载并安装到 `lib_dir`（不求解）。
pub fn sync_from_lock(
    lockfile_text: &str,
    sources: &[Source],
    lib_dir: &Path,
    download_dir: &Path,
    r_bin: &Path,
    jobs: usize,
) -> Result<Vec<String>, String> {
    let locked =
        lockfile::parse(lockfile_text).ok_or("lockfile 解析失败 / cannot parse lockfile")?;
    let plan = sync_plan(&locked, sources)?;
    run_plan(&plan, lib_dir, download_dir, r_bin, jobs)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PACKAGES: &str = "\
Package: pkgA
Version: 1.0.0

Package: pkgA
Version: 1.2.0

Package: pkgB
Version: 2.0.0
Depends: R (>= 3.0.0), pkgA (>= 1.1.0)
";

    fn one_source(repo: &str) -> Vec<Source> {
        vec![(PACKAGES.to_string(), repo.to_string())]
    }

    #[test]
    fn locks_package_and_deps() {
        let text = lock_from_packages(PACKAGES, &["pkgB".to_string()]).unwrap();
        assert!(text.contains("pkgB 2.0.0"));
        assert!(text.contains("pkgA 1.2.0"));
    }

    #[test]
    fn propagates_resolve_error() {
        let pkgs = "Package: x\nVersion: 1.0\nDepends: nope (>= 1.0)\n";
        assert!(lock_from_packages(pkgs, &["x".to_string()]).is_err());
    }

    #[test]
    fn builds_install_plan_with_urls() {
        let plan =
            install_plan(&one_source("https://repo.example"), &["pkgB".to_string()]).unwrap();
        let pkgb = plan.iter().find(|i| i.name == "pkgB").unwrap();
        assert_eq!(
            pkgb.url,
            "https://repo.example/src/contrib/pkgB_2.0.0.tar.gz"
        );
    }

    fn item(name: &str) -> InstallItem {
        InstallItem {
            name: name.to_string(),
            version: "1.0".to_string(),
            url: "u".to_string(),
            hash: String::new(),
        }
    }

    #[test]
    fn parallel_runs_f_for_every_item() {
        let plan = vec![item("a"), item("b"), item("c"), item("d")];
        let seen = std::sync::Mutex::new(std::collections::HashSet::new());
        parallel_for_each(&plan, 3, |it| {
            seen.lock().unwrap().insert(it.name.clone());
            Ok(())
        })
        .unwrap();
        let s = seen.into_inner().unwrap();
        assert_eq!(s.len(), 4); // 每个都跑到，无重无漏
        assert!(s.contains("a") && s.contains("d"));
    }

    #[test]
    fn parallel_concurrency_one_still_does_all() {
        let plan = vec![item("a"), item("b")];
        let count = std::sync::Mutex::new(0);
        parallel_for_each(&plan, 1, |_| {
            *count.lock().unwrap() += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(*count.lock().unwrap(), 2);
    }

    #[test]
    fn parallel_propagates_error() {
        let plan = vec![item("ok"), item("bad")];
        let err = parallel_for_each(&plan, 4, |it| {
            if it.name == "bad" {
                Err("boom".to_string())
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert!(err.contains("boom"));
    }

    #[test]
    fn parallel_empty_is_ok() {
        assert!(parallel_for_each(&[], 4, |_| Ok(())).is_ok());
    }

    fn idx(pkgs: &str) -> PackageIndex {
        build_index(&[(pkgs.to_string(), "https://r".to_string())])
    }

    #[test]
    fn topo_orders_chain_deps_first() {
        // C → B → A：无论输入顺序，输出都是 A、B、C。
        let index = idx(
            "Package: pkgA\nVersion: 1.0\n\nPackage: pkgB\nVersion: 1.0\nImports: pkgA\n\nPackage: pkgC\nVersion: 1.0\nImports: pkgB\n",
        );
        let got = topo_order(vec![item("pkgC"), item("pkgA"), item("pkgB")], &index);
        let order: Vec<&str> = got.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(order, ["pkgA", "pkgB", "pkgC"]);
    }

    #[test]
    fn topo_orders_diamond() {
        // D → B,C；B,C → A。A 最先、D 最后，B/C 之间按名字序。
        let index = idx(
            "Package: pkgA\nVersion: 1.0\n\nPackage: pkgB\nVersion: 1.0\nImports: pkgA\n\nPackage: pkgC\nVersion: 1.0\nImports: pkgA\n\nPackage: pkgD\nVersion: 1.0\nImports: pkgB, pkgC\n",
        );
        let got = topo_order(
            vec![item("pkgD"), item("pkgC"), item("pkgB"), item("pkgA")],
            &index,
        );
        let order: Vec<&str> = got.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(order, ["pkgA", "pkgB", "pkgC", "pkgD"]);
    }

    #[test]
    fn topo_independent_uses_name_order() {
        let index = idx("Package: pkgB\nVersion: 1.0\n\nPackage: pkgA\nVersion: 1.0\n");
        let got = topo_order(vec![item("pkgB"), item("pkgA")], &index);
        let order: Vec<&str> = got.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(order, ["pkgA", "pkgB"]);
    }

    #[test]
    fn topo_empty_index_falls_back_to_name_order() {
        // 无图（如自包含 sync 没传 --repo）→ 退化为名字序，绝不丢包。
        let got = topo_order(vec![item("z"), item("a")], &PackageIndex::default());
        let order: Vec<&str> = got.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(order, ["a", "z"]);
    }

    fn locked(ver: &str, repo: &str) -> Locked {
        Locked {
            version: Version::parse(ver).unwrap(),
            repo: repo.to_string(),
            hash: String::new(),
        }
    }

    #[test]
    fn sync_plan_installs_exact_locked_version_no_drift() {
        // 索引里有 pkgA 1.0.0 和 1.2.0；锁定 1.0.0（v1 无仓库，回退索引）→ 计划必须用 1.0.0。
        let mut lk = BTreeMap::new();
        lk.insert("pkgA".to_string(), locked("1.0.0", ""));
        let plan = sync_plan(&lk, &one_source("https://repo.example")).unwrap();
        let a = plan.iter().find(|i| i.name == "pkgA").unwrap();
        assert_eq!(a.version, "1.0.0");
        assert_eq!(a.url, "https://repo.example/src/contrib/pkgA_1.0.0.tar.gz");
    }

    #[test]
    fn sync_plan_v2_self_contained_without_sources() {
        // v2：lockfile 自带仓库 → 不传任何 --repo（sources 为空）也能拼出地址。
        let mut lk = BTreeMap::new();
        lk.insert("pkgA".to_string(), locked("1.0.0", "https://locked-repo"));
        let plan = sync_plan(&lk, &[]).unwrap();
        let a = plan.iter().find(|i| i.name == "pkgA").unwrap();
        assert_eq!(a.url, "https://locked-repo/src/contrib/pkgA_1.0.0.tar.gz");
    }

    #[test]
    fn sync_plan_errors_when_no_source_anywhere() {
        // v1 无仓库 + sources 里也没有 → 报错指名是哪个包。
        let mut lk = BTreeMap::new();
        lk.insert("pkgA".to_string(), locked("9.9.9", ""));
        let err = sync_plan(&lk, &one_source("https://repo.example")).unwrap_err();
        assert!(err.contains("pkgA"));
    }

    #[test]
    fn sync_from_lock_rejects_bad_lockfile() {
        let bad = "not a valid lockfile line";
        let err = sync_from_lock(
            bad,
            &one_source("https://r"),
            Path::new("x"),
            Path::new("y"),
            Path::new("R"),
            4,
        )
        .unwrap_err();
        assert!(err.contains("lockfile"));
    }

    #[test]
    fn install_plan_uses_each_packages_own_repo() {
        // pkgX 在 r1、依赖 pkgY；pkgY 在 r2。各自的下载地址应指向各自的仓库。
        let sources = vec![
            (
                "Package: pkgX\nVersion: 1.0\nImports: pkgY\n".to_string(),
                "https://r1".to_string(),
            ),
            (
                "Package: pkgY\nVersion: 2.0\n".to_string(),
                "https://r2".to_string(),
            ),
        ];
        let plan = install_plan(&sources, &["pkgX".to_string()]).unwrap();
        let x = plan.iter().find(|i| i.name == "pkgX").unwrap();
        let y = plan.iter().find(|i| i.name == "pkgY").unwrap();
        assert_eq!(x.url, "https://r1/src/contrib/pkgX_1.0.tar.gz");
        assert_eq!(y.url, "https://r2/src/contrib/pkgY_2.0.tar.gz");
    }
}
