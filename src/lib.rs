//! uvr —— 一个用 Rust 写的、类似 uv/Cargo 的 R 语言包管理器（教学项目）。
//!
//! 库 crate：存放可复用的核心逻辑，供二进制（`src/main.rs`）与测试使用。

pub mod commands;
pub mod fetch;
pub mod install;
pub mod lockfile;
pub mod metadata;
pub mod resolver;
pub mod version;
