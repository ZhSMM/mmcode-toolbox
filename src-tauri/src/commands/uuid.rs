//! UUID 生成。

use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppResult;
use crate::logging::log_and_run;

#[derive(Debug, Deserialize)]
pub struct UuidReq {
    #[serde(default = "default_count")]
    pub count: usize,
    #[serde(default = "default_version")]
    pub version: String, // "v4" | "v7" | "nil"
    #[serde(default)]
    pub uppercase: bool,
    #[serde(default)]
    pub with_hyphens: bool, // false 时去掉 -
}
fn default_count() -> usize { 1 }
fn default_version() -> String { "v4".to_string() }

#[derive(Debug, serde::Serialize)]
pub struct UuidResp {
    pub items: Vec<String>,
}

#[tauri::command]
pub fn uuid_generate(req: UuidReq) -> AppResult<UuidResp> {
        log_and_run("uuid_generate", "uuid", || {

    let count = req.count.clamp(1, 1000);
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let s = match req.version.as_str() {
            "v4" => Uuid::new_v4().to_string(),
            "v7" => Uuid::now_v7().to_string(),
            "nil" => Uuid::nil().to_string(),
            other => return Err(crate::error::AppError::Invalid(format!("不支持的 UUID 版本: {other}"))),
        };
        let s = if !req.with_hyphens { s.replace('-', "") } else { s };
        items.push(if req.uppercase { s.to_uppercase() } else { s });
    }
    Ok(UuidResp { items })

        })
    }