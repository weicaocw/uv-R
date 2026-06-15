//! 依赖求解（教学版）：从包索引里挑出满足约束的版本，递归解析整棵依赖树，并检测冲突。
//!
//! 注：这是便于学习的简化求解器；工业级实现（如 Rust 的 `pubgrub`）有更强的
//! 回溯与冲突解释能力，但引入它需要联网拉取 crate。这里用纯 std 自己写，便于离线学习。

use crate::metadata::{Package, PackageIndex};
use crate::version::{Constraint, Version};
use std::collections::BTreeMap;

/// 求解失败的原因。
#[derive(Debug, PartialEq, Eq)]
pub enum ResolveError {
    /// 索引里没有这个包。
    NotFound(String),
    /// 包存在，但没有版本满足约束。
    Unsatisfiable(String),
    /// 同一个包被要求了互不兼容的版本。
    Conflict(String),
}

/// 从索引中选出满足约束的"最高版本"包；约束为 `None` 表示不限版本。
///
/// 返回的引用借用自 `index`——注意签名里的生命周期 `'a`：它告诉编译器
/// "返回的 `&Package` 活得和传入的 `index` 一样久"。
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

/// 从根包出发，递归解析所有传递依赖，得到"包名 → 选定版本"。
///
/// - 跳过对 R 本身的伪依赖（`R (>= x)`）；
/// - 找不到包 / 无满足版本 / 版本冲突 → 返回对应的 [`ResolveError`]；
/// - 简化：贪心选"最高满足版本"且**不回溯**。真实求解器（pubgrub）会回溯，
///   以避开那些"换个版本本可满足"的可避免冲突。
pub fn resolve(
    index: &PackageIndex,
    root: &str,
) -> Result<BTreeMap<String, Version>, ResolveError> {
    let mut resolved: BTreeMap<String, Version> = BTreeMap::new();
    let mut queue: Vec<(String, Option<Constraint>)> = vec![(root.to_string(), None)];

    while let Some((name, constraint)) = queue.pop() {
        if name == "R" {
            continue; // R 解释器伪依赖，跳过
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
        // low 要 pkgA < 1.1.0，high 要 pkgA >= 1.2.0 —— 不可兼容
        assert_eq!(
            resolve(&idx, "root"),
            Err(ResolveError::Conflict("pkgA".to_string()))
        );
    }
}
