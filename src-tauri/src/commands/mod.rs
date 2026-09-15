//! 所有 Tauri command 的注册中心。
//!
//! 注意:`#[tauri::command]` 会在函数所在模块生成 `__cmd__<name>` 符号,
//! `tauri::generate_handler!` 用路径查找它。不要 `pub use` 重导出函数本身,
//! 否则宏定位不到 `__cmd__`。在 lib.rs 中用完整路径 `commands::<sub>::<fn>`。

pub mod base64;
pub mod favorites;
pub mod history;
pub mod json_fmt;
pub mod tools;