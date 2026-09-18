//! 随机数生成器(整数 / 浮点 / 抽样 / 骰子)。
//!
//! 用 enum holder 同时持有 `StdRng`(可种子)和 `ThreadRng`(线程本地),
//! 避免 `Rng` trait 不是 dyn-compatible 的限制。

use rand::seq::SliceRandom;
use rand::{Rng, RngCore, SeedableRng, rngs::StdRng};
use serde::Deserialize;

use crate::error::AppResult;
use crate::logging::log_and_run;

enum RngHolder {
    Seeded(StdRng),
    Threaded(rand::rngs::ThreadRng),
}

impl RngCore for RngHolder {
    fn next_u32(&mut self) -> u32 {
        match self { Self::Seeded(r) => r.next_u32(), Self::Threaded(r) => r.next_u32() }
    }
    fn next_u64(&mut self) -> u64 {
        match self { Self::Seeded(r) => r.next_u64(), Self::Threaded(r) => r.next_u64() }
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        match self { Self::Seeded(r) => r.fill_bytes(dest), Self::Threaded(r) => r.fill_bytes(dest) }
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        match self { Self::Seeded(r) => r.try_fill_bytes(dest), Self::Threaded(r) => r.try_fill_bytes(dest) }
    }
}

#[derive(Debug, Deserialize)]
pub struct RandomReq {
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub count: Option<usize>,
    #[serde(default)]
    pub items: Option<String>,
    #[serde(default)]
    pub seed: Option<u64>,
}
fn default_mode() -> String { "int".to_string() }

#[derive(Debug, serde::Serialize)]
pub struct RandomResp {
    pub results: Vec<String>,
    pub description: String,
}

#[tauri::command]
pub fn random_generate(req: RandomReq) -> AppResult<RandomResp> {
        log_and_run("random_generate", "random", || {

    let count = req.count.unwrap_or(1);
    let mut rng = if let Some(s) = req.seed {
        RngHolder::Seeded(StdRng::seed_from_u64(s))
    } else {
        RngHolder::Threaded(rand::thread_rng())
    };
    let mut results = Vec::new();
    let desc: String;
    match req.mode.as_str() {
        "int" => {
            let lo = req.min.unwrap_or(0.0) as i64;
            let hi = req.max.unwrap_or(100.0) as i64;
            if lo > hi { return Err(crate::error::AppError::Invalid("min 必须 ≤ max".into())); }
            desc = format!("整数 [{lo}, {hi})");
            for _ in 0..count {
                results.push(rng.gen_range(lo..hi).to_string());
            }
        }
        "float" => {
            let lo = req.min.unwrap_or(0.0);
            let hi = req.max.unwrap_or(1.0);
            if lo > hi { return Err(crate::error::AppError::Invalid("min 必须 ≤ max".into())); }
            desc = format!("浮点 [{lo}, {hi})");
            for _ in 0..count {
                let v: f64 = rng.gen_range(lo..hi);
                results.push(format!("{v}"));
            }
        }
        "dice" => {
            let sides = req.max.unwrap_or(6.0) as u32;
            desc = format!("{count} 个 d{sides}");
            for _ in 0..count {
                let v: u32 = rng.gen_range(1..=sides);
                results.push(v.to_string());
            }
        }
        "bytes" => {
            let n = count;
            desc = format!("{n} 字节随机 hex");
            let mut buf = vec![0u8; n];
            rng.fill_bytes(&mut buf);
            results.push(hex::encode(buf));
        }
        "pick" => {
            let items = req.items.unwrap_or_default();
            let list: Vec<&str> = items.split(|c: char| c == ',' || c == '\n').map(str::trim).filter(|s| !s.is_empty()).collect();
            if list.is_empty() { return Err(crate::error::AppError::Invalid("items 为空".into())); }
            desc = format!("从 {} 项中抽 {} 个(可重复)", list.len(), count);
            for _ in 0..count {
                results.push(list.choose(&mut rng).unwrap().to_string());
            }
        }
        "shuffle" => {
            let items = req.items.unwrap_or_default();
            let mut list: Vec<&str> = items.split(|c: char| c == ',' || c == '\n').map(str::trim).filter(|s| !s.is_empty()).collect();
            if list.is_empty() { return Err(crate::error::AppError::Invalid("items 为空".into())); }
            desc = format!("洗牌 {} 项", list.len());
            list.shuffle(&mut rng);
            results.extend(list.iter().map(|s| s.to_string()));
        }
        other => return Err(crate::error::AppError::Invalid(format!("未知 mode: {other}"))),
    }
    Ok(RandomResp { results, description: desc })

        })
    }