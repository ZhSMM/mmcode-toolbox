//! SQLite 连接管理 + 迁移 + seed。

pub mod migrate;

use std::path::Path;

use rusqlite::Connection;

use crate::error::{AppError, AppResult};

/// 打开 SQLite 连接、执行迁移、返回已就绪的连接。
pub fn open_and_migrate(path: &Path) -> AppResult<Connection> {
    let conn = Connection::open(path)?;
    // WAL 模式 + 较短的同步间隔 = 桌面工具的最佳实践
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrate::run(&conn)?;
    Ok(conn)
}

/// 用 Rust 常量 seed 工具元数据。重复调用幂等（INSERT OR IGNORE）。
pub fn seed_tools(conn: &Connection) -> AppResult<()> {
    use crate::commands::tools::{CATEGORIES, TOOLS};

    let tx = conn.unchecked_transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT OR IGNORE INTO tools (tool_id, name, category, route, icon, sort_order, enabled) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )?;
        for t in TOOLS.iter() {
            stmt.execute(rusqlite::params![
                t.tool_id,
                t.name,
                t.category,
                t.route,
                t.icon,
                t.sort_order,
                t.enabled as i32,
            ])?;
        }
    }

    // 类别表是派生数据：直接维护在常量里；不在 DB 中持久化。
    // 这里把 category 字段一致性校验一遍。
    for t in TOOLS.iter() {
        if !CATEGORIES.iter().any(|c| c.id == t.category) {
            return Err(AppError::Internal(format!(
                "工具 {} 的 category={} 在 CATEGORIES 中找不到",
                t.tool_id, t.category
            )));
        }
    }
    tx.commit()?;
    Ok(())
}