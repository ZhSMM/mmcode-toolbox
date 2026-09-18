//! 时间戳 ↔ 日期 转换。

use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::logging::log_and_run;

#[derive(Debug, Deserialize)]
pub struct TimestampParseReq {
    pub input: String,
    #[serde(default = "default_unit")]
    pub unit: String, // "s" | "ms"
}
fn default_unit() -> String { "ms".to_string() }

#[derive(Debug, serde::Serialize)]
pub struct TimestampParseResp {
    pub ms: i64,
    pub seconds: i64,
    pub iso_utc: String,
    pub iso_local: String,
    pub rfc2822: String,
    pub timezone_offset: String,
    pub weekday: String,
}

#[tauri::command]
pub fn timestamp_parse(req: TimestampParseReq) -> AppResult<TimestampParseResp> {
        log_and_run("timestamp_parse", "timestamp", || {

    let v: i64 = req.input.trim().parse()
        .map_err(|e| AppError::Invalid(format!("非法时间戳: {e}")))?;
    let ms = match req.unit.as_str() {
        "s" => v * 1000,
        "ms" => v,
        _ => return Err(AppError::Invalid(format!("未知 unit: {}", req.unit))),
    };
    let utc = DateTime::<Utc>::from_timestamp_millis(ms)
        .ok_or_else(|| AppError::Invalid("时间戳超出范围".into()))?;
    let local: DateTime<Local> = utc.with_timezone(&Local);
    let wd = ["周日","周一","周二","周三","周四","周五","周六"][local.format("%w").to_string().parse::<usize>().unwrap_or(0)];

    Ok(TimestampParseResp {
        ms,
        seconds: ms / 1000,
        iso_utc: utc.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        iso_local: local.format("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string(),
        rfc2822: local.format("%a, %d %b %Y %H:%M:%S %z").to_string(),
        timezone_offset: local.format("%:z").to_string(),
        weekday: wd.to_string(),
    })

        })
    }

#[derive(Debug, Deserialize)]
pub struct TimestampToReq {
    pub input: String,            // ISO 或人类可读
    #[serde(default)]
    pub timezone: String,         // "local" | "utc"
}

#[derive(Debug, serde::Serialize)]
pub struct TimestampToResp {
    pub ms: i64,
    pub seconds: i64,
    pub iso_utc: String,
}

#[tauri::command]
pub fn timestamp_to(req: TimestampToReq) -> AppResult<TimestampToResp> {
        log_and_run("timestamp_to", "timestamp", || {

    let s = req.input.trim();
    // 尝试 RFC3339 / ISO8601
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        let ms = dt.timestamp_millis();
        return Ok(TimestampToResp { ms, seconds: ms / 1000, iso_utc: dt.with_timezone(&Utc).format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string() });
    }
    // 常见本地格式
    let formats = ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S", "%Y/%m/%d %H:%M:%S", "%Y-%m-%d"];
    for f in &formats {
        if let Ok(ndt) = NaiveDateTime::parse_from_str(s, f) {
            let ms = match req.timezone.as_str() {
                "utc" => ndt.and_utc().timestamp_millis(),
                _ => Local.from_local_datetime(&ndt).unwrap().timestamp_millis(),
            };
            let utc = DateTime::<Utc>::from_timestamp_millis(ms).unwrap();
            return Ok(TimestampToResp { ms, seconds: ms / 1000, iso_utc: utc.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string() });
        }
    }
    Err(AppError::Invalid(format!("无法解析日期: {s}")))

        })
    }

#[derive(Debug, serde::Serialize)]
pub struct NowResp {
    pub ms: i64,
    pub seconds: i64,
    pub iso_utc: String,
    pub iso_local: String,
}

#[tauri::command]
pub fn timestamp_now() -> NowResp {
    let now = Utc::now();
    let ms = now.timestamp_millis();
    let local: DateTime<Local> = now.with_timezone(&Local);
    NowResp {
        ms, seconds: ms / 1000,
        iso_utc: now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        iso_local: local.format("%Y-%m-%dT%H:%M:%S%.3f").to_string(),
    }
}