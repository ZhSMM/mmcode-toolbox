//! 命令级别的结构化日志。
//!
//! 设计目标:
//! 1. 每个 `#[tauri::command]` 自动记录「开始 / 成功 / 失败 / 耗时」
//! 2. 错误时打印 `kind` + 完整 message,方便定位
//! 3. 默认 INFO 级别只打印摘要;DEBUG 级别打印输入/输出详情(可能含敏感数据,慎用)
//!
//! 用法:
//!
//! ```rust
//! #[tauri::command]
//! pub fn my_command(req: MyReq) -> AppResult<MyResp> {
//!     log_and_run("my_command", "my-tool", || {
//!         // 业务逻辑
//!         Ok(MyResp { .. })
//!     })
//! }
//! ```
//!
//! 或者更详细地手工控制:
//!
//! ```rust
//! #[tauri::command]
//! pub fn my_command(req: MyReq) -> AppResult<MyResp> {
//!     let _trace = CmdTrace::new("my_command", Some("my-tool"))
//!         .with_input_summary(&req.input);
//!     // ... 业务 ...
//!     match result {
//!         Ok(v) => { _trace.ok(); Ok(v) }
//!         Err(e) => { _trace.err(&e); Err(e) }
//!     }
//! }
//! ```

use std::time::Instant;

use crate::error::{AppError, AppResult};

/// 命令执行追踪器。RAII 风格:drop 时若未显式 ok/err 则记 warn。
///
/// 通常用 [`log_and_run`] 包装就够了,不需要手动管理。
pub struct CmdTrace {
    name: String,
    tool_id: Option<String>,
    input_summary: Option<String>,
    start: Instant,
    finished: bool,
}

impl CmdTrace {
    pub fn new(name: impl Into<String>, tool_id: Option<impl Into<String>>) -> Self {
        let name = name.into();
        let tool_id = tool_id.map(|s| s.into());
        log::info!(
            target: "mmcode::cmd",
            "[{}] start tool={}",
            name,
            tool_id.as_deref().unwrap_or("-")
        );
        Self {
            name,
            tool_id,
            input_summary: None,
            start: Instant::now(),
            finished: false,
        }
    }

    pub fn ok(mut self) {
        let ms = self.start.elapsed().as_millis();
        log::info!(
            target: "mmcode::cmd",
            "[{}] ok in {}ms tool={}{}",
            self.name,
            ms,
            self.tool_id.as_deref().unwrap_or("-"),
            self.input_summary
                .as_deref()
                .map(|s| format!(" input={s}"))
                .unwrap_or_default()
        );
        self.finished = true;
    }

    pub fn err(mut self, e: &AppError) {
        let ms = self.start.elapsed().as_millis();
        // 错误日志更详细:kind + 完整 message
        log::error!(
            target: "mmcode::cmd",
            "[{}] FAILED in {}ms tool={} kind={} msg=\"{}\"",
            self.name,
            ms,
            self.tool_id.as_deref().unwrap_or("-"),
            e.kind(),
            e
        );
        self.finished = true;
    }
}

impl Drop for CmdTrace {
    fn drop(&mut self) {
        if !self.finished {
            let ms = self.start.elapsed().as_millis();
            log::warn!(
                target: "mmcode::cmd",
                "[{}] trace dropped without ok/err in {}ms (可能 panic 或提前 return) tool={}",
                self.name,
                ms,
                self.tool_id.as_deref().unwrap_or("-")
            );
        }
    }
}

/// 包装 command 业务逻辑,自动记录开始/结束/失败 + 耗时。
///
/// ```ignore
/// #[tauri::command]
/// pub fn my_cmd(req: MyReq) -> AppResult<MyResp> {
///     log_and_run("my_cmd", "my-tool", || {
///         // 业务逻辑
///         Ok(MyResp { ... })
///     })
/// }
/// ```
pub fn log_and_run<F, T>(cmd: &str, tool_id: &str, f: F) -> AppResult<T>
where
    F: FnOnce() -> AppResult<T>,
{
    let trace = CmdTrace::new(cmd, Some(tool_id));
    let result = f();
    match &result {
        Ok(_) => trace.ok(),
        Err(e) => trace.err(e),
    }
    result
}

/// 包装无 tool_id 的命令(系统级:菜单 / 历史 / 收藏)。
pub fn log_and_run_sys<F, T>(cmd: &str, f: F) -> AppResult<T>
where
    F: FnOnce() -> AppResult<T>,
{
    let trace = CmdTrace::new(cmd, None::<&str>);
    let result = f();
    match &result {
        Ok(_) => trace.ok(),
        Err(e) => trace.err(e),
    }
    result
}

#[macro_export]
macro_rules! log_and_run_macro {
    ($cmd:literal, $tool_id:literal, $body:block) => {{
        $crate::log::log_and_run($cmd, $tool_id, || $body)
    }};
}