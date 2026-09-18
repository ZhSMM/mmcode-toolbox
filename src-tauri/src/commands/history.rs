//! 历史记录 CRUD。

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::logging::{log_and_run, log_and_run_sys};
use crate::state::AppState;

const INPUT_MAX: usize = 16 * 1024;     // 16KB
const OUTPUT_MAX: usize = 64 * 1024;    // 64KB

#[derive(Debug, Serialize)]
pub struct HistoryItem {
    pub id: i64,
    pub tool_id: String,
    pub input: Option<String>,
    pub output: Option<String>,
    pub status: String,
    pub error_msg: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct SaveHistoryReq {
    pub tool_id: String,
    pub input: Option<String>,
    pub output: Option<String>,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub error_msg: Option<String>,
}

fn default_status() -> String {
    "success".to_string()
}

fn truncate(s: &Option<String>, max: usize) -> Option<String> {
    s.as_ref().map(|v| {
        if v.len() <= max {
            v.clone()
        } else {
            // 截断并在末尾追加标记，方便前端识别。
            let mut out = v[..max].to_string();
            out.push_str("\n...[已截断]...");
            out
        }
    })
}

#[tauri::command]
pub fn save_history(state: tauri::State<'_, AppState>, req: SaveHistoryReq) -> AppResult<i64> {
        log_and_run_sys("save_history", || {

    if req.tool_id.is_empty() {
        return Err(AppError::Invalid("tool_id 不能为空".into()));
    }
    let now = chrono::Utc::now().timestamp_millis();
    let input = truncate(&req.input, INPUT_MAX);
    let output = truncate(&req.output, OUTPUT_MAX);

    state.with_db(|conn| {
        conn.execute(
            "INSERT INTO history (tool_id, input, output, status, error_msg, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                req.tool_id,
                input,
                output,
                req.status,
                req.error_msg,
                now,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    })

        })
    }

#[tauri::command]
pub fn list_history(
    state: tauri::State<'_, AppState>,
    tool_id: String,
    limit: u32,
) -> AppResult<Vec<HistoryItem>> {
    let limit = limit.clamp(1, 200);
    state.with_db(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, tool_id, input, output, status, error_msg, created_at \
             FROM history WHERE tool_id = ?1 ORDER BY created_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![tool_id, limit], |row| {
            Ok(HistoryItem {
                id: row.get(0)?,
                tool_id: row.get(1)?,
                input: row.get(2)?,
                output: row.get(3)?,
                status: row.get(4)?,
                error_msg: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
    })
}

#[tauri::command]
pub fn delete_history(state: tauri::State<'_, AppState>, id: i64) -> AppResult<()> {
        log_and_run_sys("delete_history", || {

    state.with_db(|conn| {
        conn.execute("DELETE FROM history WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    })

        })
    }

#[derive(Debug, Deserialize)]
pub struct ClearHistoryReq {
    #[serde(default)]
    pub tool_id: Option<String>,
}

#[tauri::command]
pub fn clear_history(state: tauri::State<'_, AppState>, req: ClearHistoryReq) -> AppResult<()> {
        log_and_run_sys("clear_history", || {

    state.with_db(|conn| {
        match req.tool_id {
            Some(tid) => {
                conn.execute("DELETE FROM history WHERE tool_id = ?1", rusqlite::params![tid])?;
            }
            None => {
                conn.execute("DELETE FROM history", [])?;
            }
        }
        Ok(())
    })

        })
    }