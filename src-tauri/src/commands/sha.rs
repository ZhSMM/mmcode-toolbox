//! SHA-1 / SHA-256 / SHA-512 散列。

use serde::Deserialize;
use sha1::{Sha1, Digest as _};
use sha2::{Sha256, Sha512};

use crate::error::AppResult;
use crate::logging::log_and_run;

#[derive(Debug, Deserialize)]
pub struct ShaReq {
    pub input: String,
    #[serde(default = "default_alg")]
    pub algorithm: String, // "sha1" | "sha256" | "sha512"
    #[serde(default)]
    pub uppercase: bool,
}
fn default_alg() -> String { "sha256".to_string() }

#[derive(Debug, serde::Serialize)]
pub struct ShaResp {
    pub hex: String,
    pub base64: String,
}

#[tauri::command]
pub fn sha_hash(req: ShaReq) -> AppResult<ShaResp> {
        log_and_run("sha_hash", "sha", || {

    let bytes = match req.algorithm.as_str() {
        "sha1" => Sha1::digest(req.input.as_bytes()).to_vec(),
        "sha256" => Sha256::digest(req.input.as_bytes()).to_vec(),
        "sha512" => Sha512::digest(req.input.as_bytes()).to_vec(),
        other => return Err(crate::error::AppError::Invalid(format!("未知算法: {other}"))),
    };
    let hex = if req.uppercase { hex::encode(&bytes).to_uppercase() } else { hex::encode(&bytes) };
    let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    Ok(ShaResp { hex, base64 })

        })
    }