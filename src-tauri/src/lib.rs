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

use tauri::Manager;

mod commands;
mod db;
mod error;
mod state;

use crate::error::AppResult;
use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志（仅一次）
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();

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
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}

/// 给单元测试用的辅助函数（不导出到生产）。
#[doc(hidden)]
pub fn _ensure_link(_: AppResult<()>) {}