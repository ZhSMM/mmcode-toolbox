//! JWT 解码(无验签 + 可选 HMAC 验签)。
//!
//! 默认模式只做 base64url 解码并展示 header + payload(不验签)。
//! `verify_hmac` 模式用 `algorithm` + `secret` 校验签名。

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::logging::{log_and_run, log_and_run_sys};

#[derive(Debug, Deserialize)]
pub struct JwtDecodeReq {
    pub token: String,
    #[serde(default)]
    pub verify: bool,
    #[serde(default)]
    pub algorithm: Option<String>, // HS256 / HS384 / HS512
    #[serde(default)]
    pub secret: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct JwtPart {
    pub raw: String,           // 原始字符串
    pub decoded: Value,        // 解析后的 JSON
}

#[derive(Debug, Serialize)]
pub struct JwtDecodeResp {
    pub header: JwtPart,
    pub payload: JwtPart,
    pub signature_hex: String,
    pub signature_bytes: usize,
    pub verified: Option<bool>, // 仅 verify 模式时返回
    pub error: Option<String>,  // 验签失败原因(不阻塞其它字段)
}

fn b64url_decode(s: &str) -> Result<Vec<u8>, AppError> {
    // JWT base64url:把 - 和 _ 替换回 + 和 /,补齐 padding
    let mut t = s.replace('-', "+").replace('_', "/");
    while t.len() % 4 != 0 { t.push('='); }
    general_purpose::STANDARD.decode(&t).map_err(|e| AppError::Invalid(format!("base64url: {e}")))
}

fn b64url_encode(bytes: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

#[tauri::command]
pub fn jwt_decode(req: JwtDecodeReq) -> AppResult<JwtDecodeResp> {
        log_and_run("jwt_decode", "jwt", || {

    let token = req.token.trim();
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(AppError::Invalid("JWT 必须由 3 段(header.payload.signature)组成".into()));
    }
    let header_bytes = b64url_decode(parts[0])?;
    let payload_bytes = b64url_decode(parts[1])?;
    let signature = b64url_decode(parts[2])?;
    let header_val: Value = serde_json::from_slice(&header_bytes)
        .map_err(|e| AppError::Invalid(format!("header JSON 解析失败: {e}")))?;
    let payload_val: Value = serde_json::from_slice(&payload_bytes)
        .map_err(|e| AppError::Invalid(format!("payload JSON 解析失败: {e}")))?;

    let (verified, verify_err) = if req.verify {
        let alg = req.algorithm.unwrap_or_else(|| {
            header_val.get("alg").and_then(|v| v.as_str()).unwrap_or("HS256").to_string()
        });
        let secret = req.secret.ok_or_else(|| AppError::Invalid("verify 模式需要提供 secret".into()))?;
        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let mut verified = false;
        let mut err: Option<String> = None;
        match alg.as_str() {
            "HS256" => {
                use hmac::{Hmac, Mac};
                use sha2::Sha256;
                type HmacSha256 = Hmac<Sha256>;
                match HmacSha256::new_from_slice(secret.as_bytes()) {
                    Ok(mut mac) => {
                        mac.update(signing_input.as_bytes());
                        let expect = mac.finalize().into_bytes();
                        verified = expect.as_slice() == signature.as_slice();
                        if !verified { err = Some("HMAC-SHA256 签名不匹配".into()); }
                    }
                    Err(e) => err = Some(format!("HMAC 初始化失败: {e}")),
                }
            }
            "HS384" => {
                use hmac::{Hmac, Mac};
                use sha2::Sha384;
                type HmacSha384 = Hmac<Sha384>;
                match HmacSha384::new_from_slice(secret.as_bytes()) {
                    Ok(mut mac) => {
                        mac.update(signing_input.as_bytes());
                        let expect = mac.finalize().into_bytes();
                        verified = expect.as_slice() == signature.as_slice();
                        if !verified { err = Some("HMAC-SHA384 签名不匹配".into()); }
                    }
                    Err(e) => err = Some(format!("HMAC 初始化失败: {e}")),
                }
            }
            "HS512" => {
                use hmac::{Hmac, Mac};
                use sha2::Sha512;
                type HmacSha512 = Hmac<Sha512>;
                match HmacSha512::new_from_slice(secret.as_bytes()) {
                    Ok(mut mac) => {
                        mac.update(signing_input.as_bytes());
                        let expect = mac.finalize().into_bytes();
                        verified = expect.as_slice() == signature.as_slice();
                        if !verified { err = Some("HMAC-SHA512 签名不匹配".into()); }
                    }
                    Err(e) => err = Some(format!("HMAC 初始化失败: {e}")),
                }
            }
            other => err = Some(format!("不支持的算法: {other} (仅支持 HS256/384/512)")),
        }
        (Some(verified), err)
    } else {
        (None, None)
    };

    Ok(JwtDecodeResp {
        header: JwtPart { raw: parts[0].to_string(), decoded: header_val },
        payload: JwtPart { raw: parts[1].to_string(), decoded: payload_val },
        signature_hex: hex::encode(&signature),
        signature_bytes: signature.len(),
        verified,
        error: verify_err,
    })

        })
    }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decode_unsigned() {
        // 经典示例 jwt.io header.payload.signature
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let r = jwt_decode(JwtDecodeReq { token: token.into(), verify: false, algorithm: None, secret: None }).unwrap();
        assert_eq!(r.payload.decoded["sub"], "1234567890");
    }
}