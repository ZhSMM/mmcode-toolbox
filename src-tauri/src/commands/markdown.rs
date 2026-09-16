//! Markdown → HTML 渲染(pulldown-cmark)。

use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;

use crate::error::AppResult;

#[derive(Debug, Deserialize)]
pub struct MarkdownReq {
    pub input: String,
    #[serde(default = "default_options")]
    pub enable_tables: bool,
    #[serde(default = "default_options")]
    pub enable_footnotes: bool,
    #[serde(default = "default_options")]
    pub enable_strikethrough: bool,
    #[serde(default = "default_options")]
    pub enable_tasklists: bool,
}
fn default_options() -> bool { true }

#[derive(Debug, serde::Serialize)]
pub struct MarkdownResp {
    pub html: String,
    pub byte_size: usize,
    pub line_count: usize,
}

#[tauri::command]
pub fn markdown_render(req: MarkdownReq) -> AppResult<MarkdownResp> {
    let mut opts = Options::empty();
    if req.enable_tables { opts.insert(Options::ENABLE_TABLES); }
    if req.enable_footnotes { opts.insert(Options::ENABLE_FOOTNOTES); }
    if req.enable_strikethrough { opts.insert(Options::ENABLE_STRIKETHROUGH); }
    if req.enable_tasklists { opts.insert(Options::ENABLE_TASKLISTS); }

    let parser = Parser::new_ext(&req.input, opts);
    let mut output = String::with_capacity(req.input.len());
    html::push_html(&mut output, parser);

    let line_count = if req.input.is_empty() { 0 } else { req.input.matches('\n').count() + 1 };
    Ok(MarkdownResp {
        html: output,
        byte_size: req.input.len(),
        line_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let r = markdown_render(MarkdownReq {
            input: "# Hello\n\nThis is **bold** and *italic*.\n\n- a\n- b\n".into(),
            enable_tables: false, enable_footnotes: false, enable_strikethrough: false, enable_tasklists: false,
        }).unwrap();
        assert!(r.html.contains("<h1>"));
        assert!(r.html.contains("<strong>"));
    }
}