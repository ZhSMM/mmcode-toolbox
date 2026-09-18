//! JSON 格式化 + 压缩。
//!
//! 纯函数实现，不依赖第三方 crate 之外的 `serde_json`。

use serde::Deserialize;
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::logging::log_and_run;

#[derive(Debug, Deserialize)]
pub struct JsonReq {
    pub input: String,
    #[serde(default = "default_indent")]
    pub indent: usize,
    #[serde(default)]
    pub sort_keys: bool,
    #[serde(default)]
    pub escape_unicode: bool,
}

fn default_indent() -> usize {
    2
}

#[derive(Debug, serde::Serialize)]
pub struct JsonResp {
    pub output: String,
    pub duration_ms: u128,
}

/// 格式化（pretty print）。
#[tauri::command]
pub fn json_format(req: JsonReq) -> AppResult<JsonResp> {
        log_and_run("json_format", "json_fmt", || {

    let started = std::time::Instant::now();
    let input = req.input.trim();
    if input.is_empty() {
        return Err(AppError::Invalid("输入为空".into()));
    }
    if input.len() > 4 * 1024 * 1024 {
        return Err(AppError::Invalid("输入超过 4MB 上限".into()));
    }

    let mut value: Value = serde_json::from_str(input)?;
    if req.sort_keys {
        sort_value_keys(&mut value);
    }

    let indent = req.indent.clamp(0, 8);
    let indent_str = " ".repeat(indent * 2); // 每级 2 空格为基础；每多一级再翻倍

    let mut output = if indent == 0 {
        // indent=0 表示压缩到单行
        serde_json::to_string(&value)?
    } else {
        let indent_bytes = indent_str.as_bytes();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(indent_bytes);
        let mut buf = Vec::with_capacity(input.len() * 2);
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        use serde::Serialize;
        value.serialize(&mut ser)?;
        String::from_utf8(buf).map_err(|e| AppError::Internal(e.to_string()))?
    };

    // escape_unicode: 把非 ASCII 字符转成 \uXXXX
    if req.escape_unicode {
        output = escape_unicode_in_string(&output);
    }

    Ok(JsonResp {
        output,
        duration_ms: started.elapsed().as_millis(),
    })

        })
    }

/// 压缩（minify）。
#[tauri::command]
pub fn json_minify(req: JsonReq) -> AppResult<JsonResp> {
        log_and_run("json_minify", "json_fmt", || {

    let started = std::time::Instant::now();
    let input = req.input.trim();
    if input.is_empty() {
        return Err(AppError::Invalid("输入为空".into()));
    }
    if input.len() > 4 * 1024 * 1024 {
        return Err(AppError::Invalid("输入超过 4MB 上限".into()));
    }
    let mut value: Value = serde_json::from_str(input)?;
    if req.sort_keys {
        sort_value_keys(&mut value);
    }
    let mut output = serde_json::to_string(&value)?;
    if req.escape_unicode {
        output = escape_unicode_in_string(&output);
    }
    Ok(JsonResp {
        output,
        duration_ms: started.elapsed().as_millis(),
    })

        })
    }

/// 递归对 Object 的键排序。
fn sort_value_keys(v: &mut Value) {
    match v {
        Value::Object(map) => {
            // 取出 entries，按 key 排序，重新插入到新 Map 中
            let mut entries: Vec<(String, Value)> = std::mem::take(map)
                .into_iter()
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            // 递归处理 value
            for (_, val) in entries.iter_mut() {
                sort_value_keys(val);
            }
            let sorted = entries.into_iter().collect();
            *map = sorted;
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                sort_value_keys(item);
            }
        }
        _ => {}
    }
}

/// 把 JSON 字符串字面量内部的非 ASCII 字符转义为 \uXXXX。
/// 仅在格式化输出上做一次扫描，识别 `"..."` 字符串并替换内容。
fn escape_unicode_in_string(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    let mut in_str = false;
    let mut escape = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            if escape {
                // 已有反斜杠转义，原样输出当前字符
                out.push(b as char);
                escape = false;
            } else if b == b'\\' {
                out.push('\\');
                escape = true;
            } else if b == b'"' {
                out.push('"');
                in_str = false;
            } else if b >= 0x80 {
                // 非 ASCII → 转 \uXXXX
                // 取完整 UTF-8 序列
                let ch_start = i;
                let ch = decode_utf8_char(bytes, &mut i);
                for c in ch.escape_unicode().to_string().chars() {
                    out.push(c);
                }
                // i 已被 decode 推进
                let _ = ch_start;
                continue;
            } else {
                out.push(b as char);
            }
        } else {
            out.push(b as char);
            if b == b'"' {
                in_str = true;
            }
        }
        i += 1;
    }
    out
}

fn decode_utf8_char(bytes: &[u8], i: &mut usize) -> char {
    let b0 = bytes[*i];
    let len = if b0 < 0x80 {
        1
    } else if b0 < 0xC0 {
        1
    } else if b0 < 0xE0 {
        2
    } else if b0 < 0xF0 {
        3
    } else {
        4
    };
    let slice = &bytes[*i..(*i + len).min(bytes.len())];
    let s = std::str::from_utf8(slice).unwrap_or("?");
    let c = s.chars().next().unwrap_or('?');
    *i += len - 1; // 外层还会 +1
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let r = json_format(JsonReq {
            input: r#"{"b":1,"a":2}"#.into(),
            indent: 2,
            sort_keys: true,
            escape_unicode: false,
        })
        .unwrap();
        assert!(r.output.contains("\"a\": 2"));
        assert!(r.output.contains("\"b\": 1"));
    }

    #[test]
    fn test_invalid() {
        let r = json_format(JsonReq {
            input: "not json".into(),
            indent: 2,
            sort_keys: false,
            escape_unicode: false,
        });
        assert!(r.is_err());
    }
}