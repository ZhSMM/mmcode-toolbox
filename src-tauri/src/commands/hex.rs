//! Hex(十六进制) 转换与查看。
//!
//! 提供三种模式:
//! - text → hex: UTF-8 字节序列的 hex 字符串
//! - hex → text: hex 字符串还原为文本
//! - hexdump: 文本按字节 hexdump(`0000  48 65 6c 6c 6f ...  |Hello.|`)便于肉眼检视

use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::logging::{log_and_run, log_and_run_sys};

#[derive(Debug, Deserialize)]
pub struct HexReq {
    pub input: String,
    #[serde(default = "default_mode")]
    pub mode: String, // "encode" | "decode" | "hexdump"
    #[serde(default)]
    pub uppercase: bool,
}

fn default_mode() -> String { "encode".to_string() }

#[derive(Debug, serde::Serialize)]
pub struct HexResp {
    pub output: String,
}

#[tauri::command]
pub fn hex_encode(req: HexReq) -> AppResult<HexResp> {
        log_and_run("hex_encode", "hex", || {

    let bytes = req.input.as_bytes();
    let mut s = hex::encode(bytes);
    if req.uppercase { s = s.to_uppercase(); }
    Ok(HexResp { output: s })

        })
    }

#[tauri::command]
pub fn hex_decode(req: HexReq) -> AppResult<HexResp> {
        log_and_run("hex_decode", "hex", || {

    let cleaned: String = req.input.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = hex::decode(&cleaned)
        .map_err(|e| AppError::Invalid(format!("非法 hex: {e}")))?;
    let s = String::from_utf8(bytes)
        .map_err(|e| AppError::Invalid(format!("解码后不是合法 UTF-8: {e}")))?;
    Ok(HexResp { output: s })

        })
    }

#[tauri::command]
pub fn hex_dump(req: HexReq) -> AppResult<HexResp> {
        log_and_run("hex_dump", "hex", || {

    let bytes = req.input.as_bytes();
    let mut out = String::new();
    for (i, chunk) in bytes.chunks(16).enumerate() {
        let offset = format!("{:08x}  ", i * 16);
        let mut hex = String::new();
        let mut ascii = String::new();
        for (j, b) in chunk.iter().enumerate() {
            if j == 8 { hex.push(' '); }
            hex.push_str(&format!("{:02x} ", b));
            ascii.push(if (0x20..0x7f).contains(b) { *b as char } else { '.' });
        }
        // 不足 16 字节时填充
        if chunk.len() < 16 {
            let pad = (16 - chunk.len()) * 3 + if chunk.len() <= 8 { 1 } else { 0 };
            for _ in 0..pad { hex.push(' '); }
        }
        let line = if req.uppercase {
            format!("{}{} |{}|\n", offset, hex.to_uppercase(), ascii)
        } else {
            format!("{offset}{hex} |{ascii}|\n")
        };
        out.push_str(&line);
    }
    Ok(HexResp { output: out })

        })
    }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encode_decode() {
        let r = hex_encode(HexReq { input: "Hi".into(), mode: "encode".into(), uppercase: false }).unwrap();
        assert_eq!(r.output, "4869");
        let d = hex_decode(HexReq { input: "4869".into(), mode: "decode".into(), uppercase: false }).unwrap();
        assert_eq!(d.output, "Hi");
    }
    #[test]
    fn dump() {
        let r = hex_dump(HexReq { input: "Hello, world!".into(), mode: "hexdump".into(), uppercase: false }).unwrap();
        // dump 输出带空格(48 65 6c 6c 6f 是 "Hello")
        assert!(r.output.contains("48 65 6c 6c 6f"));
    }
}