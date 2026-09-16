//! 所有 Tauri command 的注册中心。
//!
//! `#[tauri::command]` 在函数所在模块生成 `__cmd__<name>`,lib.rs 用完整路径
//! `commands::<sub>::<fn>` 引用,以便 `tauri::generate_handler!` 能找到。

pub mod aes;
pub mod base64;
pub mod base_n;
pub mod cidr;
pub mod color;
pub mod cron;
pub mod csv_viewer;
pub mod diff;
pub mod favorites;
pub mod hex;
pub mod history;
pub mod hmac;
pub mod json_fmt;
pub mod jwt;
pub mod markdown;
pub mod md5;
pub mod password;
pub mod qrcode;
pub mod random;
pub mod regex;
pub mod sha;
pub mod sql_format;
pub mod string_stats;
pub mod timestamp;
pub mod tools;
pub mod url_codec;
pub mod uuid;