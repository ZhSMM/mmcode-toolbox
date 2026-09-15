//! 全局共享状态。当前只有 SQLite 连接；后续可加缓存、配置等。

use std::sync::Mutex;

use rusqlite::Connection;

pub struct AppState {
    pub db: Mutex<Connection>,
}

impl AppState {
    /// 获取 db 锁的便捷方法。
    pub fn with_db<F, R>(&self, f: F) -> Result<R, crate::error::AppError>
    where
        F: FnOnce(&mut Connection) -> Result<R, rusqlite::Error>,
    {
        let mut guard = self
            .db
            .lock()
            .map_err(|e| crate::error::AppError::Internal(format!("db lock poisoned: {e}")))?;
        f(&mut guard).map_err(crate::error::AppError::from)
    }
}