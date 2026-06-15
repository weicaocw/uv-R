//! 依赖求解：一个手写的教学版（贪心、不回溯）与一个接 `pubgrub` 的工业版（回溯）。

use crate::metadata::{Package, PackageIndex};
use crate::version::{Constraint, Op, Version};
use pubgrub::{OfflineDependencyProvider, Ranges};
use std::collections::BTreeMap;

/// 求解失败的原因。
#[derive(Debug, PartialEq, Eq)]
pub enum ResolveError {
    /// 索引里没有这个包。
    NotFound(String),
    /// 包存在，但没有版本满足约束。
    Unsatisfiable(String),
    /// 同一个包被要求了互不兼容的版本（贪心求解器的判断）。
    Conflict(String),
    /// pubgrub 求解器报告无解（含其推导说明）。
    NoSolution(String),
}

/// 随 R 一起发行的 base / recommended 包：它们不在 `PACKAGES` 里、也无需单独安装，求解时跳过。
const BUILTIN: &[&str] = &[
    // base
    "R",
    "base",
    "compiler",
    "datasets",
    "graphics",
    "grDevices",
    "grid",
    "methods",
    "parallel",
    "splines",
    "stats",
    "stats4",
    "tcltk",
    "tools",
    "utils",
    "translations",
    // recommended（随 R 发行）
    "boot",
    "class",
    "cluster",
    "codetools",
    "foreign",
    "KernSmooth",
    "lattice",
    "MASS",
    "Matrix",
    "mgcv",
    "nlme",
    "nnet",
    "rpart",
    "spatial",
    "survival",
];

/// 是否是随 R 自带、无需安装的包。
pub fn is_builtin(name: &str) -> bool {
    BUILTIN.contains(&name)
}

/// 从索引中选出满足约束的"最高版本"包；约束为 `None` 表示不限版本。
pub fn best_match<'a>(
    index: &'a PackageIndex,
    name: &str,
    constraint: Option<&Constraint>,
) -> Option<&'a Package> {
    index
        .versions_of(name)
        .iter()
        .filter(|pkg| constraint.is_none_or(|c| c.matches(&pkg.version)))
        .max_by(|a, b| a.version.cmp(&b.version))
}

/// 手写教学版求解器：从根包出发递归解析所有传递依赖，**贪心选最高满足版本、不回溯**。
///
/// - 跳过随 R 自带的 base / recommended 包；
/// - 找不到包 / 无满足版本 / 版本冲突 → 返回对应的 [`ResolveError`]；
/// - 因为不回溯，可能把"换个版本本可满足"的情况误报为冲突——这正是工业版用
///   [`resolve_pubgrub`] 的意义。
pub fn resolve(
    index: &PackageIndex,
    root: &str,
) -> Result<BTreeMap<String, Version>, ResolveError> {
    let mut resolved: BTreeMap<String, Version> = BTreeMap::new();
    let mut queue: Vec<(String, Option<Constraint>)> = vec![(root.to_string(), None)];

    while let Some((name, constraint)) = queue.pop() {
        if is_builtin(&name) {
            continue; // 随 R 自带的包，无需求解 / 安装
        }

        // 已解析过：检查新约束是否与已选版本兼容，不兼容即冲突
        if let Some(chosen) = resolved.get(&name) {
            if let Some(c) = &constraint
                && !c.matches(chosen)
            {
                return Err(ResolveError::Conflict(name));
            }
            continue;
        }

        // 尚未解析：先确认包存在，再选满足约束的最高版本
        if index.versions_of(&name).is_empty() {
            return Err(ResolveError::NotFound(name));
        }
        let pkg = best_match(index, &name, constraint.as_ref())
            .ok_or_else(|| ResolveError::Unsatisfiable(name.clone()))?;
        resolved.insert(name, pkg.version.clone());
        for dep in &pkg.depends {
            queue.push((dep.name.clone(), dep.constraint.clone()));
        }
    }
    Ok(resolved)
}

/// 把我们的 `Constraint` 转成 pubgrub 的版本集合 `Ranges`。
fn constraint_to_range(constraint: Option<&Constraint>) -> Ranges<Version> {
    match constraint {
        None => Ranges::full(),
        Some(c) => match c.op() {
            Op::Ge => Ranges::higher_than(c.version().clone()),
            Op::Gt => Ranges::strictly_higher_than(c.version().clone()),
            Op::Le => Ranges::lower_than(c.version().clone()),
            Op::Lt => Ranges::strictly_lower_than(c.version().clone()),
            Op::Eq => Ranges::singleton(c.version().clone()),
        },
    }
}

/// 工业版求解器：把依赖图灌进 pubgrub 的 `OfflineDependencyProvider`，调用 `pubgrub::resolve`。
///
/// 相比 [`resolve`]，pubgrub **会回溯**，能解开"贪心会误判为冲突"的情形。
pub fn resolve_pubgrub(
    index: &PackageIndex,
    root: &str,
) -> Result<BTreeMap<String, Version>, ResolveError> {
    let mut dp = OfflineDependencyProvider::<String, Ranges<Version>>::new();
    for pkg in index.packages() {
        let deps: Vec<(String, Ranges<Version>)> = pkg
            .depends
            .iter()
            .filter(|d| !is_builtin(&d.name))
            .map(|d| (d.name.clone(), constraint_to_range(d.constraint.as_ref())))
            .collect();
        dp.add_dependencies(pkg.name.clone(), pkg.version.clone(), deps);
    }

    // pubgrub 从一个具体的 (根包, 根版本) 出发；取根包的最高可用版本。
    let root_version = best_match(index, root, None)
        .ok_or_else(|| ResolveError::NotFound(root.to_string()))?
        .version
        .clone();

    match pubgrub::resolve(&dp, root.to_string(), root_version) {
        Ok(solution) => Ok(solution.into_iter().collect()),
        Err(e) => Err(ResolveError::NoSolution(format!("{e:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
Package: pkgA
Version: 1.0.0

Package: pkgA
Version: 1.2.0

Package: pkgB
Version: 2.0.0
Depends: R (>= 3.0.0), pkgA (>= 1.1.0)
";

    fn index() -> PackageIndex {
        PackageIndex::from_packages_file(SAMPLE)
    }

    #[test]
    fn picks_highest_when_unconstrained() {
        let idx = index();
        let p = best_match(&idx, "pkgA", None).unwrap();
        assert_eq!(p.version, Version::parse("1.2.0").unwrap());
    }

    #[test]
    fn respects_constraint() {
        let idx = index();
        let c = Constraint::parse("< 1.1.0").unwrap();
        let p = best_match(&idx, "pkgA", Some(&c)).unwrap();
        assert_eq!(p.version, Version::parse("1.0.0").unwrap());
    }

    #[test]
    fn none_when_missing_or_unsatisfiable() {
        let idx = index();
        assert!(best_match(&idx, "missing", None).is_none());
        let c = Constraint::parse(">= 9.0").unwrap();
        assert!(best_match(&idx, "pkgA", Some(&c)).is_none());
    }

    #[test]
    fn resolves_transitive_deps() {
        let idx = index();
        let res = resolve(&idx, "pkgB").unwrap();
        assert_eq!(res["pkgB"], Version::parse("2.0.0").unwrap());
        assert_eq!(res["pkgA"], Version::parse("1.2.0").unwrap());
        assert!(!res.contains_key("R"));
    }

    #[test]
    fn skips_builtin_packages() {
        let idx = PackageIndex::from_packages_file(
            "Package: pkgB\nVersion: 2.0.0\nDepends: R (>= 3.0.0), utils, methods\n",
        );
        let res = resolve(&idx, "pkgB").unwrap();
        assert!(!res.contains_key("utils"));
        assert!(!res.contains_key("methods"));
        assert_eq!(res["pkgB"], Version::parse("2.0.0").unwrap());
    }

    #[test]
    fn missing_dependency_is_not_found() {
        let idx =
            PackageIndex::from_packages_file("Package: x\nVersion: 1.0\nDepends: nope (>= 1.0)\n");
        assert_eq!(
            resolve(&idx, "x"),
            Err(ResolveError::NotFound("nope".to_string()))
        );
    }

    const CONFLICT_SAMPLE: &str = "\
Package: pkgA
Version: 1.0.0

Package: pkgA
Version: 1.2.0

Package: low
Version: 1.0.0
Depends: pkgA (< 1.1.0)

Package: high
Version: 1.0.0
Depends: pkgA (>= 1.2.0)

Package: root
Version: 1.0.0
Depends: low, high
";

    #[test]
    fn detects_version_conflict() {
        let idx = PackageIndex::from_packages_file(CONFLICT_SAMPLE);
        assert_eq!(
            resolve(&idx, "root"),
            Err(ResolveError::Conflict("pkgA".to_string()))
        );
    }

    // —— pubgrub 工业版 ——

    #[test]
    fn pubgrub_agrees_with_handwritten() {
        let idx = index();
        let a = resolve(&idx, "pkgB").unwrap();
        let b = resolve_pubgrub(&idx, "pkgB").unwrap();
        assert_eq!(a, b); // 无冲突的简单图上，两者给出同一个解
    }

    // 一个"贪心会误判冲突、但其实有解"的图：root 依赖 B、A；A 有 1.0/2.0；
    // B 要求 A < 2.0。贪心先把 A 选成最高 2.0 → 撞 B 的 A<2 → 报冲突；
    // pubgrub 回溯把 A 选成 1.0 → 有解。
    const BACKTRACK_SAMPLE: &str = "\
Package: A
Version: 1.0.0

Package: A
Version: 2.0.0

Package: B
Version: 1.0.0
Depends: A (< 2.0.0)

Package: root
Version: 1.0.0
Depends: B, A
";

    #[test]
    fn pubgrub_backtracks_where_greedy_conflicts() {
        let idx = PackageIndex::from_packages_file(BACKTRACK_SAMPLE);
        // 贪心求解器：误报冲突
        assert!(matches!(
            resolve(&idx, "root"),
            Err(ResolveError::Conflict(_))
        ));
        // pubgrub：回溯成功，A 选 1.0.0
        let sol = resolve_pubgrub(&idx, "root").unwrap();
        assert_eq!(sol["A"], Version::parse("1.0.0").unwrap());
        assert_eq!(sol["B"], Version::parse("1.0.0").unwrap());
    }
}
