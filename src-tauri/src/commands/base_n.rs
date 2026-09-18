//! Base32 / Base58 编解码。

use data_encoding::{Encoding, BASE32, BASE32HEX, BASE32_NOPAD, BASE32HEX_NOPAD};
use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::logging::{log_and_run, log_and_run_sys};

fn pick_encoding(name: &str, no_pad: bool) -> Encoding {
    let n = name.to_ascii_lowercase();
    if !no_pad {
        if n == "base32" { return BASE32; }
        if n == "base32hex" { return BASE32HEX; }
    } else {
        if n == "base32" { return BASE32_NOPAD; }
        if n == "base32hex" { return BASE32HEX_NOPAD; }
    }
    BASE32 // fallback, 调用方会通过 decode 失败来报错
}

fn check_encoding(name: &str, no_pad: bool) -> Result<(), String> {
    let n = name.to_ascii_lowercase();
    let valid = if no_pad { n == "base32" || n == "base32hex" } else { n == "base32" || n == "base32hex" };
    if valid { Ok(()) } else { Err(format!("未知 base32 变体: {name}")) }
}

#[derive(Debug, Deserialize)]
pub struct Base32Req {
    pub input: String,
    #[serde(default = "default_variant")]
    pub variant: String,
    #[serde(default)]
    pub no_pad: bool,
}
fn default_variant() -> String { "base32".to_string() }

#[derive(Debug, Deserialize)]
pub struct Base58Req {
    pub input: String,
}

#[derive(Debug, serde::Serialize)]
pub struct Base32Resp {
    pub encoded: String,
}

#[derive(Debug, serde::Serialize)]
pub struct Base58Resp {
    pub encoded: String,
}

#[tauri::command]
pub fn base32_encode(req: Base32Req) -> AppResult<Base32Resp> {
        log_and_run("base32_encode", "base-n", || {

    check_encoding(&req.variant, req.no_pad).map_err(AppError::Invalid)?;
    let enc = pick_encoding(&req.variant, req.no_pad);
    Ok(Base32Resp { encoded: enc.encode(req.input.as_bytes()) })

        })
    }

#[tauri::command]
pub fn base32_decode(req: Base32Req) -> AppResult<Base32Resp> {
        log_and_run("base32_decode", "base-n", || {

    check_encoding(&req.variant, req.no_pad).map_err(AppError::Invalid)?;
    let enc = pick_encoding(&req.variant, req.no_pad);
    let bytes = enc.decode(req.input.as_bytes()).map_err(|e| AppError::Invalid(format!("base32 解码失败: {e}")))?;
    let s = String::from_utf8(bytes).map_err(|e| AppError::Invalid(format!("解码后不是合法 UTF-8: {e}")))?;
    Ok(Base32Resp { encoded: s })

        })
    }

#[tauri::command]
pub fn base58_encode(req: Base58Req) -> AppResult<Base58Resp> {
        log_and_run("base58_encode", "base-n", || {

    Ok(Base58Resp { encoded: bs58::encode(req.input.as_bytes()).into_string() })

        })
    }

#[tauri::command]
pub fn base58_decode(req: Base58Req) -> AppResult<Base58Resp> {
        log_and_run("base58_decode", "base-n", || {

    let bytes = bs58::decode(req.input.trim()).into_vec().map_err(|e| AppError::Invalid(format!("base58 解码失败: {e}")))?;
    let s = String::from_utf8(bytes).map_err(|e| AppError::Invalid(format!("解码后不是合法 UTF-8: {e}")))?;
    Ok(Base58Resp { encoded: s })

        })
    }