//! Base64 编解码。
//!
//! 用 enum + 同一个 concrete `GeneralPurpose` 引擎返回类型,避免 `Box<dyn Engine>`
//! 必须指定两个关联类型的麻烦。`STANDARD` / `URL_SAFE_NO_PAD` 都是 `GeneralPurpose`。

use base64::{engine::general_purpose::GeneralPurpose, Engine as _};
use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::logging::{log_and_run, log_and_run_sys};

#[derive(Debug, Deserialize)]
pub struct Base64Req {
    pub input: String,
    #[serde(default)]
    pub url_safe: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct Base64Resp {
    pub output: String,
}

fn pick_engine(url_safe: bool) -> GeneralPurpose {
    if url_safe {
        base64::engine::general_purpose::URL_SAFE_NO_PAD
    } else {
        base64::engine::general_purpose::STANDARD
    }
}

#[tauri::command]
pub fn base64_encode(req: Base64Req) -> AppResult<Base64Resp> {
        log_and_run("base64_encode", "base64", || {

    if req.input.len() > 4 * 1024 * 1024 {
        return Err(AppError::Invalid("输入超过 4MB 上限".into()));
    }
    let engine = pick_engine(req.url_safe);
    Ok(Base64Resp {
        output: engine.encode(req.input.as_bytes()),
    })

        })
    }

#[tauri::command]
pub fn base64_decode(req: Base64Req) -> AppResult<Base64Resp> {
        log_and_run("base64_decode", "base64", || {

    if req.input.len() > 8 * 1024 * 1024 {
        return Err(AppError::Invalid("输入超过 8MB 上限".into()));
    }
    let engine = pick_engine(req.url_safe);
    let bytes = engine
        .decode(req.input.trim())
        .map_err(|e| AppError::Invalid(format!("Base64 解码失败: {e}")))?;
    let output = String::from_utf8(bytes)
        .map_err(|e| AppError::Invalid(format!("解码结果不是合法 UTF-8: {e}")))?;
    Ok(Base64Resp { output })

        })
    }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let enc = base64_encode(Base64Req { input: "hello".into(), url_safe: false }).unwrap();
        assert_eq!(enc.output, "aGVsbG8=");
        let dec = base64_decode(Base64Req { input: enc.output, url_safe: false }).unwrap();
        assert_eq!(dec.output, "hello");
    }

    #[test]
    fn test_url_safe() {
        let enc = base64_encode(Base64Req { input: "????".into(), url_safe: true }).unwrap();
        // URL-safe 不使用 padding
        assert!(!enc.output.ends_with('='));
    }

    #[test]
    fn test_invalid() {
        let r = base64_decode(Base64Req { input: "!!!".into(), url_safe: false });
        assert!(r.is_err());
    }
}