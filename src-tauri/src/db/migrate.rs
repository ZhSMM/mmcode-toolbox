//! 简易迁移器：扫描 `migrations/` 目录、按文件名字典序执行。
//!
//! 不引第三方 migrate 库，避免拖重依赖。生产用法：每次启动检查已执行版本。
//! 当前实现：每次启动执行所有 *.sql，依赖 `CREATE ... IF NOT EXISTS` 幂等。

use rusqlite::Connection;

use crate::error::AppResult;

pub fn run(conn: &Connection) -> AppResult<()> {
    // 把 migrations/*.sql 内联编译进来，避免运行时再去读文件系统（生产打包后路径会变）。
    let migrations: &[(&str, &str)] = &[("001_init", include_str!("../../migrations/001_init.sql"))];

    for (name, sql) in migrations {
        log::info!("执行迁移: {name}");
        conn.execute_batch(sql)?;
    }
    Ok(())
}