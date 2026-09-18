//! 随机密码生成。

use rand::Rng;
use serde::Deserialize;

use crate::error::AppResult;
use crate::logging::{log_and_run, log_and_run_sys};

#[derive(Debug, Deserialize)]
pub struct PasswordReq {
    #[serde(default = "default_len")]
    pub length: usize,
    #[serde(default = "default_count")]
    pub count: usize,
    #[serde(default = "default_true")]
    pub uppercase: bool,
    #[serde(default = "default_true")]
    pub lowercase: bool,
    #[serde(default = "default_true")]
    pub digits: bool,
    #[serde(default)]
    pub symbols: bool,
    #[serde(default)]
    pub no_ambiguous: bool, // 去掉 0 O o I l 1
    #[serde(default)]
    pub must_include_each: bool, // 保证每类至少出现一次
}

fn default_len() -> usize { 16 }
fn default_count() -> usize { 1 }
fn default_true() -> bool { true }

const UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()-_=+[]{};:,.<>/?";
const AMBIGUOUS: &str = "0OoIl1|`'\" ";

#[derive(Debug, serde::Serialize)]
pub struct PasswordResp {
    pub passwords: Vec<String>,
    pub strength: String, // "weak" / "medium" / "strong" (基于熵估计)
    pub entropy_bits: f64,
}

#[tauri::command]
pub fn password_generate(req: PasswordReq) -> AppResult<PasswordResp> {
        log_and_run("password_generate", "password", || {

    let length = req.length.clamp(4, 256);
    let count = req.count.clamp(1, 50);
    let mut alphabet = String::new();
    let mut classes: Vec<&str> = vec![];
    if req.uppercase { alphabet.push_str(UPPER); classes.push(UPPER); }
    if req.lowercase { alphabet.push_str(LOWER); classes.push(LOWER); }
    if req.digits { alphabet.push_str(DIGITS); classes.push(DIGITS); }
    if req.symbols { alphabet.push_str(SYMBOLS); classes.push(SYMBOLS); }
    if req.no_ambiguous {
        alphabet = alphabet.chars().filter(|c| !AMBIGUOUS.contains(*c)).collect();
        classes = classes.iter().map(|s| s.to_string()).collect::<Vec<_>>().iter().map(|s| s.chars().filter(|c| !AMBIGUOUS.contains(*c)).collect::<String>().leak() as &str).collect();
    }
    if alphabet.is_empty() {
        return Err(crate::error::AppError::Invalid("至少启用一种字符集".into()));
    }
    let mut rng = rand::thread_rng();
    let alphabet: Vec<char> = alphabet.chars().collect();
    let mut passwords = Vec::with_capacity(count);
    for _ in 0..count {
        let pwd = if req.must_include_each && classes.iter().all(|c| !c.is_empty()) {
            // 先每类取 1 个,再随机补足
            let mut chars: Vec<char> = classes.iter().map(|s| {
                let v: Vec<char> = s.chars().collect();
                v[rng.gen_range(0..v.len())]
            }).collect();
            while chars.len() < length {
                chars.push(alphabet[rng.gen_range(0..alphabet.len())]);
            }
            // 洗牌
            use rand::seq::SliceRandom;
            chars.shuffle(&mut rng);
            chars.into_iter().collect()
        } else {
            (0..length).map(|_| alphabet[rng.gen_range(0..alphabet.len())]).collect()
        };
        passwords.push(pwd);
    }
    // 熵估计 = length * log2(alphabet_size)
    let entropy = (length as f64) * (alphabet.len() as f64).log2();
    let strength = if entropy < 40.0 { "弱" } else if entropy < 72.0 { "中" } else { "强" }.to_string();
    Ok(PasswordResp { passwords, strength, entropy_bits: (entropy * 10.0).round() / 10.0 })

        })
    }