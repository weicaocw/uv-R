//! 依赖求解（教学版）：从包索引里挑出满足约束的版本。
//!
//! 注：这是便于学习的简化求解器；工业级实现（如 Rust 的 `pubgrub`）有更强的
//! 回溯与冲突解释能力，但引入它需要联网拉取 crate。这里用纯 std 自己写，便于离线学习。

use crate::metadata::{Package, PackageIndex};
use crate::version::Constraint;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::Version;

    const SAMPLE: &str = "\
Package: pkgA
Version: 1.0.0

Package: pkgA
Version: 1.2.0

Package: pkgB
Version: 2.0.0
Depends: pkgA (>= 1.1.0)
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
}
