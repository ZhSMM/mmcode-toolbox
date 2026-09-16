//! URL 编解码(percent-encoding)。

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::Deserialize;

use crate::error::{AppError, AppResult};

/// 应用到查询字符串的字符集(空格 → %20)。
/// RFC 3986 unreserved 字符集 = ALPHA / DIGIT / "-" / "." / "_" / "~"
const QUERY_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ').add(b'"').add(b'#').add(b'$').add(b'%').add(b'&')
    .add(b'\'').add(b'(').add(b')').add(b'*').add(b'+').add(b',')
    .add(b'/').add(b':').add(b';').add(b'<').add(b'=').add(b'>')
    .add(b'?').add(b'@').add(b'[').add(b'\\').add(b']').add(b'^')
    .add(b'`').add(b'{').add(b'|').add(b'}');

const COMPONENT_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ').add(b'"').add(b'#').add(b'$').add(b'%').add(b'&')
    .add(b'\'').add(b'(').add(b')').add(b'*').add(b'+').add(b',')
    .add(b'/').add(b':').add(b';').add(b'<').add(b'=').add(b'>')
    .add(b'?').add(b'@').add(b'[').add(b'\\').add(b']').add(b'^')
    .add(b'`').add(b'{').add(b'|').add(b'}');

#[derive(Debug, Deserialize)]
pub struct UrlCodecReq {
    pub input: String,
    /// 编码集:`query`(查询参数,空格→%20) 或 `component`(路径段,空格→%20)
    #[serde(default = "default_mode")]
    pub mode: String,
}

fn default_mode() -> String { "query".to_string() }

#[derive(Debug, serde::Serialize)]
pub struct UrlCodecResp {
    pub output: String,
}

#[tauri::command]
pub fn url_encode(req: UrlCodecReq) -> AppResult<UrlCodecResp> {
    if req.input.is_empty() {
        return Err(AppError::Invalid("输入为空".into()));
    }
    let set = match req.mode.as_str() {
        "query" => QUERY_ENCODE_SET,
        "component" => COMPONENT_ENCODE_SET,
        _ => return Err(AppError::Invalid(format!("未知 mode: {}", req.mode))),
    };
    Ok(UrlCodecResp { output: utf8_percent_encode(&req.input, set).to_string() })
}

#[tauri::command]
pub fn url_decode(req: UrlCodecReq) -> AppResult<UrlCodecResp> {
    if req.input.is_empty() {
        return Err(AppError::Invalid("输入为空".into()));
    }
    let bytes = percent_encoding::percent_decode_str(&req.input).decode_utf8()
        .map_err(|e| AppError::Invalid(format!("百分号编码非法: {e}")))?;
    Ok(UrlCodecResp { output: bytes.into_owned() })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let enc = url_encode(UrlCodecReq { input: "hello world 你好".into(), mode: "query".into() }).unwrap();
        let dec = url_decode(UrlCodecReq { input: enc.output.clone(), mode: "query".into() }).unwrap();
        assert_eq!(dec.output, "hello world 你好");
        assert!(enc.output.contains("%20"));
    }
}