//! AES-256-GCM 加解密。
//!
//! 输入约定:key(32 字节)、nonce(12 字节) 都用 hex 字符串(64 / 24 hex 字符)。
//! 输出密文 = nonce(12) || ciphertext_with_tag,统一用 hex。
//! 也可附加 AAD(additional authenticated data)以 hex 传入。

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::RngCore;
use serde::Deserialize;

use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct AesReq {
    pub input: String,
    pub key_hex: String,
    #[serde(default)]
    pub nonce_hex: Option<String>,
    #[serde(default)]
    pub aad_hex: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct AesResp {
    pub output: String,
}

fn parse_hex_bytes(s: &str, label: &str) -> Result<Vec<u8>, AppError> {
    let cleaned: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    hex::decode(&cleaned).map_err(|e| AppError::Invalid(format!("{label} hex 解码失败: {e}")))
}

fn aad_bytes(req: &AesReq) -> Result<Vec<u8>, AppError> {
    match req.aad_hex.as_deref() {
        Some(s) if !s.is_empty() => parse_hex_bytes(s, "aad"),
        _ => Ok(Vec::new()),
    }
}

#[tauri::command]
pub fn aes_encrypt(req: AesReq) -> AppResult<AesResp> {
    let key_bytes = parse_hex_bytes(&req.key_hex, "key")?;
    if key_bytes.len() != 32 {
        return Err(AppError::Invalid(format!("key 长度需 32 字节,实际 {}", key_bytes.len())));
    }
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));

    let nonce_bytes = match req.nonce_hex.as_deref() {
        Some(s) if !s.is_empty() => {
            let n = parse_hex_bytes(s, "nonce")?;
            if n.len() != 12 {
                return Err(AppError::Invalid(format!("nonce 长度需 12 字节,实际 {}", n.len())));
            }
            n
        }
        _ => vec![0u8; 12],
    };
    let nonce = Nonce::from_slice(&nonce_bytes);
    let aad = aad_bytes(&req)?;
    let payload = Payload { msg: req.input.as_bytes(), aad: &aad };

    let ciphertext = cipher.encrypt(nonce, payload)
        .map_err(|e| AppError::Internal(format!("加密失败: {e}")))?;
    let mut out = Vec::with_capacity(12 + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(AesResp { output: hex::encode(out) })
}

#[tauri::command]
pub fn aes_decrypt(req: AesReq) -> AppResult<AesResp> {
    let key_bytes = parse_hex_bytes(&req.key_hex, "key")?;
    if key_bytes.len() != 32 {
        return Err(AppError::Invalid(format!("key 长度需 32 字节,实际 {}", key_bytes.len())));
    }
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));

    let blob = parse_hex_bytes(&req.input, "input")?;
    if blob.len() < 12 + 16 {
        return Err(AppError::Invalid("密文至少需要 nonce(12)+tag(16)=28 字节".into()));
    }
    let (nonce_bytes_default, ciphertext) = blob.split_at(12);

    let nonce_bytes = match req.nonce_hex.as_deref() {
        Some(s) if !s.is_empty() => {
            let n = parse_hex_bytes(s, "nonce")?;
            if n.len() != 12 {
                return Err(AppError::Invalid(format!("nonce 长度需 12 字节,实际 {}", n.len())));
            }
            n
        }
        _ => nonce_bytes_default.to_vec(),
    };
    let nonce = Nonce::from_slice(&nonce_bytes);
    let aad = aad_bytes(&req)?;
    let payload = Payload { msg: ciphertext, aad: &aad };

    let plaintext = cipher.decrypt(nonce, payload)
        .map_err(|_| AppError::Invalid("解密失败:key/nonce/AAD 不匹配或密文被篡改".into()))?;
    let s = String::from_utf8(plaintext)
        .map_err(|e| AppError::Invalid(format!("明文不是合法 UTF-8: {e}")))?;
    Ok(AesResp { output: s })
}

#[tauri::command]
pub fn aes_random_key() -> AesResp {
    let mut key = [0u8; 32];
    let mut nonce = [0u8; 12];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut key);
    rng.fill_bytes(&mut nonce);
    AesResp { output: serde_json::json!({ "key": hex::encode(key), "nonce": hex::encode(nonce) }).to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let k = hex::encode([1u8; 32]);
        let n = hex::encode([2u8; 12]);
        let enc = aes_encrypt(AesReq {
            input: "secret".into(), key_hex: k.clone(), nonce_hex: Some(n.clone()), aad_hex: None,
        }).unwrap();
        let dec = aes_decrypt(AesReq {
            input: enc.output.clone(), key_hex: k, nonce_hex: Some(n), aad_hex: None,
        }).unwrap();
        assert_eq!(dec.output, "secret");
    }
}