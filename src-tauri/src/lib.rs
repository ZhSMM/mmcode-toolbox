//! Mavis Code Toolbox 库入口。
//!
//! 启动流程：
//! 1. 初始化日志
//! 2. 解析 SQLite 数据目录并打开连接
//! 3. 执行 migrations/*.sql 建表
//! 4. 用 Rust 常量 seed 工具元数据
//! 5. 注册 Tauri commands + plugins
//! 6. 启动应用

use std::sync::Mutex;

use log::Level;
use tauri::Manager;

mod commands;
mod db;
mod error;
mod logging;
mod state;

use crate::error::AppResult;
use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志:详细格式(时间戳 + 级别 + target),彩色输出。
    // 通过 RUST_LOG 环境变量控制级别,例如:
    //   RUST_LOG=debug pnpm tauri:dev      # 打印所有 debug
    //   RUST_LOG=mmcode_toolbox_lib=debug  # 仅本项目 debug
    let _ = env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .format(|buf, record| {
        use std::io::Write;
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let level_color = match record.level() {
            Level::Error => "\x1b[31m",  // 红
            Level::Warn  => "\x1b[33m",  // 黄
            Level::Info  => "\x1b[32m",  // 绿
            Level::Debug => "\x1b[36m",  // 青
            Level::Trace => "\x1b[37m",  // 灰
        };
        let reset = "\x1b[0m";
        writeln!(
            buf,
            "{ts} {level_color}{level:>5}{reset} [{target}] {msg}",
            level = record.level(),
            target = record.target(),
            msg = record.args(),
        )
    })
    .try_init();

    log::info!("=== Mavis Code Toolbox starting ===");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 解析应用数据目录
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("解析 app_data_dir 失败: {e}"))?;
            std::fs::create_dir_all(&app_data_dir)
                .map_err(|e| format!("创建数据目录失败: {e}"))?;
            let db_path = app_data_dir.join("app.db");
            log::info!("SQLite 数据目录: {}", db_path.display());

            // 打开 + 迁移 + seed
            let conn = db::open_and_migrate(&db_path)
                .map_err(|e| format!("初始化数据库失败: {e}"))?;
            db::seed_tools(&conn)
                .map_err(|e| format!("seed 工具元数据失败: {e}"))?;
            log::info!("数据库就绪");

            // 注入全局 state
            app.manage(AppState {
                db: Mutex::new(conn),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::tools::list_categories,
            commands::tools::list_tools,
            commands::history::list_history,
            commands::history::save_history,
            commands::history::delete_history,
            commands::history::clear_history,
            commands::favorites::list_favorites,
            commands::favorites::toggle_favorite,
            commands::json_fmt::json_format,
            commands::json_fmt::json_minify,
            commands::base64::base64_encode,
            commands::base64::base64_decode,
            // ===== 占位转正 12 个 =====
            commands::url_codec::url_encode,
            commands::url_codec::url_decode,
            commands::hex::hex_encode,
            commands::hex::hex_decode,
            commands::hex::hex_dump,
            commands::regex::regex_test,
            commands::regex::regex_replace,
            commands::diff::diff_text,
            commands::string_stats::string_stats,
            commands::md5::md5_hash,
            commands::sha::sha_hash,
            commands::hmac::hmac_hash,
            commands::uuid::uuid_generate,
            commands::password::password_generate,
            commands::timestamp::timestamp_parse,
            commands::timestamp::timestamp_to,
            commands::timestamp::timestamp_now,
            commands::qrcode::qrcode_generate,
            // ===== 发散新增 =====
            commands::aes::aes_encrypt,
            commands::aes::aes_decrypt,
            commands::aes::aes_random_key,
            commands::jwt::jwt_decode,
            commands::markdown::markdown_render,
            commands::csv_viewer::csv_parse,
            commands::csv_viewer::csv_extract_column,
            commands::color::color_convert,
            commands::cron::cron_next,
            commands::sql_format::sql_format,
            commands::base_n::base32_encode,
            commands::base_n::base32_decode,
            commands::base_n::base58_encode,
            commands::base_n::base58_decode,
            commands::cidr::cidr_info,
            commands::cidr::ip_info,
            commands::random::random_generate,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}

/// 给单元测试用的辅助函数（不导出到生产）。
#[doc(hidden)]
pub fn _ensure_link(_: AppResult<()>) {}