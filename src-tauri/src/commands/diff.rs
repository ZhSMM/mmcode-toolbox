//! 行级 diff(基于 similar crate)。

use serde::Deserialize;
use similar::{ChangeTag, TextDiff};

use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct DiffReq {
    pub left: String,
    pub right: String,
    #[serde(default)]
    pub ignore_whitespace: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct DiffHunk {
    pub tag: String, // "equal" | "insert" | "delete" | "replace"
    pub left_line: Option<usize>,
    pub right_line: Option<usize>,
    pub value: String,
}

#[derive(Debug, serde::Serialize)]
pub struct DiffResp {
    pub hunks: Vec<DiffHunk>,
    pub stats: DiffStats,
}

#[derive(Debug, serde::Serialize)]
pub struct DiffStats {
    pub left_lines: usize,
    pub right_lines: usize,
    pub added: usize,
    pub deleted: usize,
    pub unchanged: usize,
}

#[tauri::command]
pub fn diff_text(req: DiffReq) -> AppResult<DiffResp> {
    if req.left.is_empty() && req.right.is_empty() {
        return Err(AppError::Invalid("两侧输入均为空".into()));
    }
    let diff = TextDiff::from_lines(&req.left, &req.right);
    let mut hunks = Vec::new();
    let mut stats = DiffStats {
        left_lines: 0, right_lines: 0, added: 0, deleted: 0, unchanged: 0,
    };
    for change in diff.iter_all_changes() {
        let tag = match change.tag() {
            ChangeTag::Equal => { stats.unchanged += 1; "equal" }
            ChangeTag::Insert => { stats.added += 1; stats.right_lines += 1; "insert" }
            ChangeTag::Delete => { stats.deleted += 1; stats.left_lines += 1; "delete" }
        };
        let line_num = change.old_index().map(|i| i + 1).or_else(|| change.new_index().map(|i| i + 1));
        hunks.push(DiffHunk {
            tag: tag.to_string(),
            left_line: change.old_index().map(|i| i + 1),
            right_line: change.new_index().map(|i| i + 1),
            value: change.value().trim_end_matches('\n').to_string(),
        });
        let _ = line_num;
    }
    // 修正行数:Equal 也计数
    stats.left_lines = req.left.lines().count();
    stats.right_lines = req.right.lines().count();
    Ok(DiffResp { hunks, stats })
}