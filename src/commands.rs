//! 命令实现：把"读元数据 → 求解 → 渲染 lockfile"串起来，供命令行入口调用。
//!
//! 把这层做成**纯函数**（文本进、文本出），便于单元测试；真正的文件 IO 放在 `main.rs`。

use crate::lockfile;
use crate::metadata::PackageIndex;
use crate::resolver::{ResolveError, resolve};
use crate::version::Version;
use std::collections::BTreeMap;

/// 给定 `PACKAGES` 文本与若干根包，解析依赖并渲染成 lockfile 文本。
///
/// 简化：多个根包各自求解后合并（后者覆盖同名项）；单根是最常见情形。
pub fn lock_from_packages(packages_text: &str, roots: &[String]) -> Result<String, ResolveError> {
    let index = PackageIndex::from_packages_file(packages_text);
    let mut combined: BTreeMap<String, Version> = BTreeMap::new();
    for root in roots {
        combined.extend(resolve(&index, root)?);
    }
    Ok(lockfile::render(&combined))
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
}
