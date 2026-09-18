//! MD5 散列。

use md5::{Md5, Digest};
use serde::Deserialize;

use crate::error::AppResult;
use crate::logging::log_and_run;

#[derive(Debug, Deserialize)]
pub struct HashReq {
    pub input: String,
}

#[derive(Debug, serde::Serialize)]
pub struct HashResp {
    pub hex: String,
    pub base64: String,
    pub bytes: Vec<u8>,
}

#[tauri::command]
pub fn md5_hash(req: HashReq) -> AppResult<HashResp> {
    log_and_run("md5_hash", "md5", || {
        let hasher = Md5::new();
        let bytes = hasher.chain_update(req.input.as_bytes()).finalize();
        Ok(HashResp {
            hex: hex::encode(bytes),
            base64: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes),
            bytes: bytes.to_vec(),
        })
    })
}