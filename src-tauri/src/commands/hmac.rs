//! HMAC-SHA256 / SHA512。

use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::{Sha256, Sha512};

use crate::error::{AppError, AppResult};
use crate::logging::{log_and_run, log_and_run_sys};

#[derive(Debug, Deserialize)]
pub struct HmacReq {
    pub input: String,
    pub secret: String,
    #[serde(default = "default_alg")]
    pub algorithm: String, // "hmac-sha256" | "hmac-sha512"
    #[serde(default)]
    pub uppercase: bool,
}
fn default_alg() -> String { "hmac-sha256".to_string() }

#[derive(Debug, serde::Serialize)]
pub struct HmacResp {
    pub hex: String,
    pub base64: String,
}

#[tauri::command]
pub fn hmac_hash(req: HmacReq) -> AppResult<HmacResp> {
        log_and_run("hmac_hash", "hmac", || {

    let bytes = match req.algorithm.as_str() {
        "hmac-sha256" => {
            type H = Hmac<Sha256>;
            let mut mac = <H as Mac>::new_from_slice(req.secret.as_bytes())
                .map_err(|e| AppError::Invalid(format!("HMAC 初始化失败: {e}")))?;
            mac.update(req.input.as_bytes());
            mac.finalize().into_bytes().to_vec()
        }
        "hmac-sha512" => {
            type H = Hmac<Sha512>;
            let mut mac = <H as Mac>::new_from_slice(req.secret.as_bytes())
                .map_err(|e| AppError::Invalid(format!("HMAC 初始化失败: {e}")))?;
            mac.update(req.input.as_bytes());
            mac.finalize().into_bytes().to_vec()
        }
        other => return Err(AppError::Invalid(format!("未知算法: {other}"))),
    };
    let hex = if req.uppercase { hex::encode(&bytes).to_uppercase() } else { hex::encode(&bytes) };
    let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    Ok(HmacResp { hex, base64 })

        })
    }