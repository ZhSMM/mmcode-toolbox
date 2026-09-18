//! 收藏。

use crate::error::AppResult;
use crate::logging::log_and_run_sys;
use crate::state::AppState;

#[tauri::command]
pub fn list_favorites(state: tauri::State<'_, AppState>) -> AppResult<Vec<String>> {
    state.with_db(|conn| {
        let mut stmt = conn.prepare("SELECT tool_id FROM favorites ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect::<Result<Vec<_>, _>>()
    })
}

#[tauri::command]
pub fn toggle_favorite(state: tauri::State<'_, AppState>, tool_id: String) -> AppResult<bool> {
        log_and_run_sys("toggle_favorite", || {

    if tool_id.is_empty() {
        return Err(crate::error::AppError::Invalid("tool_id 不能为空".into()));
    }
    state.with_db(|conn| {
        // 已收藏则移除；否则插入
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM favorites WHERE tool_id = ?1)",
                rusqlite::params![tool_id],
                |row| row.get(0),
            )?;
        if exists {
            conn.execute("DELETE FROM favorites WHERE tool_id = ?1", rusqlite::params![tool_id])?;
            Ok(false)
        } else {
            let now = chrono::Utc::now().timestamp_millis();
            conn.execute(
                "INSERT INTO favorites (tool_id, created_at) VALUES (?1, ?2)",
                rusqlite::params![tool_id, now],
            )?;
            Ok(true)
        }
    })

        })
    }