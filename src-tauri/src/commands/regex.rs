//! 正则测试。

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::logging::log_and_run;

#[derive(Debug, Deserialize)]
pub struct RegexReq {
    pub pattern: String,
    pub input: String,
    #[serde(default)]
    pub flags: Vec<String>, // "i" / "m" / "s" / "x" / "u"
}

#[derive(Debug, Serialize)]
pub struct RegexMatch {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub groups: Vec<Option<String>>,
    pub named_groups: std::collections::HashMap<String, Option<String>>,
}

#[derive(Debug, Serialize)]
pub struct RegexResp {
    pub valid: bool,
    pub error: Option<String>,
    pub matches: Vec<RegexMatch>,
}

fn build_regex(pattern: &str, flags: &[String]) -> Result<Regex, String> {
    let mut p = String::new();
    let mut in_flags = String::new();
    for f in flags {
        match f.as_str() {
            "i" => in_flags.push('i'),
            "m" => in_flags.push('m'),
            "s" => in_flags.push('s'),
            "x" => in_flags.push('x'),
            "u" => in_flags.push('u'),
            _ => {}
        }
    }
    if !in_flags.is_empty() {
        p.push_str(&format!("(?{in_flags})"));
    }
    p.push_str(pattern);
    Regex::new(&p).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn regex_test(req: RegexReq) -> AppResult<RegexResp> {
        log_and_run("regex_test", "regex-tester", || {

    let re = match build_regex(&req.pattern, &req.flags) {
        Ok(r) => r,
        Err(e) => {
            return Ok(RegexResp { valid: false, error: Some(e), matches: vec![] });
        }
    };

    let mut out = Vec::new();
    for cap in re.captures_iter(&req.input) {
        let m = cap.get(0).unwrap();
        let mut groups = Vec::new();
        for i in 1..cap.len() {
            groups.push(cap.get(i).map(|g| g.as_str().to_string()));
        }
        let mut named = std::collections::HashMap::new();
        for name in re.capture_names().flatten() {
            named.insert(name.to_string(), cap.name(name).map(|g| g.as_str().to_string()));
        }
        out.push(RegexMatch {
            start: m.start(),
            end: m.end(),
            text: m.as_str().to_string(),
            groups,
            named_groups: named,
        });
    }
    Ok(RegexResp { valid: true, error: None, matches: out })

        })
    }

#[tauri::command]
pub fn regex_replace(pattern: String, input: String, replacement: String, flags: Vec<String>) -> AppResult<String> {
        log_and_run("regex_replace", "regex-tester", || {

    let re = build_regex(&pattern, &flags).map_err(AppError::Invalid)?;
    Ok(re.replace_all(&input, replacement.as_str()).into_owned())

        })
    }