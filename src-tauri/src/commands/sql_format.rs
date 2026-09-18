//! SQL 美化 / 压缩(sqlformat crate)。

use serde::Deserialize;

use crate::error::AppResult;
use crate::logging::{log_and_run, log_and_run_sys};

#[derive(Debug, Deserialize)]
pub struct SqlFormatReq {
    pub input: String,
    #[serde(default = "default_indent")]
    pub indent: usize,
    #[serde(default = "default_case")]
    pub uppercase: bool,
    #[serde(default)]
    pub minify: bool,
}
fn default_indent() -> usize { 2 }
fn default_case() -> bool { true }

#[derive(Debug, serde::Serialize)]
pub struct SqlFormatResp {
    pub output: String,
    pub line_count: usize,
}

#[tauri::command]
pub fn sql_format(req: SqlFormatReq) -> AppResult<SqlFormatResp> {
        log_and_run("sql_format", "sql-format", || {

    let mut options = sqlformat::FormatOptions::default();
    options.indent = sqlformat::Indent::Spaces(req.indent.clamp(1, 8) as u8);
    options.uppercase = req.uppercase;

    let output = if req.minify {
        sqlformat::format(&req.input, &sqlformat::QueryParams::None, options)
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        sqlformat::format(&req.input, &sqlformat::QueryParams::None, options)
    };
    let line_count = if output.is_empty() { 0 } else { output.matches('\n').count() + 1 };
    Ok(SqlFormatResp { output, line_count })

        })
    }