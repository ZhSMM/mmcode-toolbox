//! Cron 表达式解析 + 下次运行时间预测。
//!
//! 支持标准 5 字段:分 时 日 月 周
//! 字段值支持:`*` `N` `N-M` `N,M` `*/N`

use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use serde::Deserialize;
use std::str::FromStr;

use crate::error::{AppError, AppResult};
use crate::logging::{log_and_run, log_and_run_sys};

#[derive(Debug, Clone, PartialEq)]
enum CronField {
    Any,
    Single(i32),
    Range(i32, i32),
    Step(i32, i32),
    List(Vec<CronField>),
}

impl CronField {
    fn matches(&self, value: i32) -> bool {
        match self {
            CronField::Any => true,
            CronField::Single(v) => *v == value,
            CronField::Range(s, e) => value >= *s && value <= *e,
            CronField::Step(s, step) => value >= *s && (value - s) % step == 0,
            CronField::List(items) => items.iter().any(|f| f.matches(value)),
        }
    }

    fn min(&self) -> i32 {
        match self {
            CronField::Any => 0,
            CronField::Single(v) => *v,
            CronField::Range(s, _) => *s,
            CronField::Step(s, _) => *s,
            CronField::List(items) => items.iter().map(|f| f.min()).min().unwrap_or(0),
        }
    }
}

fn parse_field(s: &str, min: i32, max: i32) -> Result<CronField, AppError> {
    let _ = (min, max);
    if s.trim() == "*" {
        return Ok(CronField::Any);
    }
    let mut parts = Vec::new();
    for token in s.split(',') {
        if token.contains('/') {
            let (base, step_str) = token.split_once('/').unwrap();
            let step: i32 = step_str.parse().map_err(|_e: std::num::ParseIntError| AppError::Invalid("步长非法".into()))?;
            if step <= 0 { return Err(AppError::Invalid("步长必须 > 0".into())); }
            let base_field = if base == "*" {
                CronField::Any
            } else if base.contains('-') {
                let (s, e) = base.split_once('-').unwrap();
                let s: i32 = s.parse().map_err(|e: std::num::ParseIntError| AppError::Invalid(e.to_string()))?;
                let e: i32 = e.parse().map_err(|e: std::num::ParseIntError| AppError::Invalid(e.to_string()))?;
                CronField::Range(s, e)
            } else {
                let v: i32 = base.parse().map_err(|e: std::num::ParseIntError| AppError::Invalid(e.to_string()))?;
                CronField::Single(v)
            };
            let start = base_field.min();
            parts.push(CronField::Step(start, step));
        } else if token.contains('-') {
            let (s, e) = token.split_once('-').unwrap();
            let s: i32 = s.parse().map_err(|e: std::num::ParseIntError| AppError::Invalid(e.to_string()))?;
            let e: i32 = e.parse().map_err(|e: std::num::ParseIntError| AppError::Invalid(e.to_string()))?;
            parts.push(CronField::Range(s, e));
        } else {
            let v: i32 = token.parse().map_err(|_e: std::num::ParseIntError| AppError::Invalid(format!("字段值非法: {token}")))?;
            parts.push(CronField::Single(v));
        }
    }
    if parts.len() == 1 { Ok(parts.into_iter().next().unwrap()) } else { Ok(CronField::List(parts)) }
}

#[derive(Debug)]
pub struct CronExpr {
    minute: CronField,
    hour: CronField,
    day: CronField,
    month: CronField,
    weekday: CronField,
}

impl FromStr for CronExpr {
    type Err = AppError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let fields: Vec<&str> = s.split_whitespace().collect();
        if fields.len() != 5 {
            return Err(AppError::Invalid(format!("cron 需 5 字段,实际 {}", fields.len())));
        }
        Ok(CronExpr {
            minute: parse_field(fields[0], 0, 59)?,
            hour: parse_field(fields[1], 0, 23)?,
            day: parse_field(fields[2], 1, 31)?,
            month: parse_field(fields[3], 1, 12)?,
            weekday: parse_field(fields[4], 0, 6)?,
        })
    }
}

impl CronExpr {
    fn matches(&self, dt: &DateTime<Utc>) -> bool {
        let m = dt.minute() as i32;
        let h = dt.hour() as i32;
        let d = dt.day() as i32;
        let mo = dt.month() as i32;
        let wd = dt.weekday().num_days_from_sunday() as i32;
        self.minute.matches(m) && self.hour.matches(h) && self.month.matches(mo)
            && (self.day.matches(d) || self.weekday.matches(wd))
    }
}

fn next_after(expr: &CronExpr, start: DateTime<Utc>, max_search: usize) -> Option<DateTime<Utc>> {
    let mut dt = start + Duration::minutes(1);
    dt = dt.with_second(0).unwrap().with_nanosecond(0).unwrap();
    for _ in 0..max_search {
        if expr.matches(&dt) {
            return Some(dt);
        }
        dt += Duration::minutes(1);
        if (dt - start).num_days() > 366 { return None; }
    }
    None
}

#[derive(Debug, Deserialize)]
pub struct CronReq {
    pub expression: String,
    #[serde(default)]
    pub start_iso: Option<String>,
    #[serde(default = "default_count")]
    pub count: usize,
}
fn default_count() -> usize { 5 }

#[derive(Debug, serde::Serialize)]
pub struct CronResp {
    pub valid: bool,
    pub error: Option<String>,
    pub next_runs: Vec<String>,
}

#[tauri::command]
pub fn cron_next(req: CronReq) -> AppResult<CronResp> {
        log_and_run("cron_next", "cron", || {

    let expr: CronExpr = match req.expression.parse() {
        Ok(e) => e,
        Err(e) => return Ok(CronResp { valid: false, error: Some(e.to_string()), next_runs: vec![] }),
    };
    let start = match req.start_iso.as_deref() {
        Some(s) => DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
        None => Utc::now(),
    };
    let count = req.count.clamp(1, 20);
    let mut runs = Vec::with_capacity(count);
    let mut cursor = start;
    for _ in 0..count {
        match next_after(&expr, cursor, 60 * 24 * 400) {
            Some(t) => {
                runs.push(t.format("%Y-%m-%dT%H:%M:%SZ").to_string());
                cursor = t;
            }
            None => break,
        }
    }
    Ok(CronResp { valid: true, error: None, next_runs: runs })

        })
    }