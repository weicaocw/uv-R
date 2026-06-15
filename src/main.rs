//! `uvr` 命令行入口（薄壳）：调用库 crate `uvr` 里的逻辑。

use uvr::version::{Constraint, Version};

fn main() {
    let c = Constraint::parse(">= 1.2.0").unwrap();
    let v = Version::parse("1.10.0").unwrap();
    println!("{:?} 满足 >= 1.2.0 ? {}", v, c.matches(&v));
}
