//! 字符串统计:字符 / 字节 / 行 / 词 / 词频 / 字符分类。

use serde::Deserialize;
use std::collections::HashMap;

use crate::error::AppResult;

#[derive(Debug, Deserialize)]
pub struct StringStatsReq {
    pub input: String,
    #[serde(default = "default_top")]
    pub top_n: usize,
}
fn default_top() -> usize { 10 }

#[derive(Debug, serde::Serialize)]
pub struct StringStatsResp {
    pub chars: usize,
    pub chars_no_whitespace: usize,
    pub bytes: usize,
    pub lines: usize,
    pub words: usize,
    pub sentences: usize,
    pub paragraphs: usize,
    pub chinese_chars: usize,
    pub ascii_chars: usize,
    pub digits: usize,
    pub spaces: usize,
    pub top_words: Vec<(String, usize)>,
    pub byte_distribution: Vec<(u8, usize)>, // (字节值 0-255, 出现次数)
}

#[tauri::command]
pub fn string_stats(req: StringStatsReq) -> AppResult<StringStatsResp> {
    let s = &req.input;

    let chars = s.chars().count();
    let chars_no_whitespace = s.chars().filter(|c| !c.is_whitespace()).count();
    let bytes = s.len();
    let lines = if s.is_empty() { 0 } else { s.matches('\n').count() + 1 };

    // 词数:中英文混排,英文按空白分,中文按字
    let mut words = 0usize;
    let mut chinese = 0usize;
    let mut ascii = 0usize;
    let mut digits = 0usize;
    let mut spaces = 0usize;
    let mut freq: HashMap<String, usize> = HashMap::new();
    let mut in_word = false;
    let mut word_buf = String::new();
    for c in s.chars() {
        if c.is_whitespace() {
            spaces += 1;
            if in_word && !word_buf.is_empty() {
                *freq.entry(word_buf.clone()).or_insert(0) += 1;
                word_buf.clear();
                words += 1;
            }
            in_word = false;
        } else if c.is_ascii_alphabetic() {
            ascii += 1;
            in_word = true;
            word_buf.push(c);
        } else if c.is_ascii_digit() {
            digits += 1;
            in_word = true;
            word_buf.push(c);
        } else {
            // 中文/其他字符按"字"计数,各算一个"词"
            if ('\u{4e00}'..='\u{9fff}').contains(&c) { chinese += 1; }
            if in_word && !word_buf.is_empty() {
                *freq.entry(word_buf.clone()).or_insert(0) += 1;
                word_buf.clear();
                words += 1;
            }
            in_word = false;
            *freq.entry(c.to_string()).or_insert(0) += 1;
        }
    }
    if in_word && !word_buf.is_empty() {
        *freq.entry(word_buf.clone()).or_insert(0) += 1;
        words += 1;
    }
    words += chinese;

    // 句子 / 段落:基于标点和空行
    let sentences = s.matches(|c: char| c == '.' || c == '!' || c == '?' || c == '。' || c == '！' || c == '？').count();
    let paragraphs = s.split("\n\n").filter(|p| !p.trim().is_empty()).count().max(if s.contains('\n') { 1 } else { 0 });

    // top words
    let mut top: Vec<(String, usize)> = freq.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    top.truncate(req.top_n);

    // 字节分布
    let mut dist = [0usize; 256];
    for &b in s.as_bytes() { dist[b as usize] += 1; }
    let byte_distribution: Vec<(u8, usize)> = dist.iter().enumerate().filter(|(_, &c)| c > 0).map(|(b, c)| (b as u8, *c)).collect();

    Ok(StringStatsResp {
        chars, chars_no_whitespace, bytes, lines, words, sentences, paragraphs,
        chinese_chars: chinese, ascii_chars: ascii, digits, spaces,
        top_words: top, byte_distribution,
    })
}