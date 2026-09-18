//! CSV 工具:解析、统计、列提取。

use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::logging::{log_and_run, log_and_run_sys};

#[derive(Debug, Deserialize)]
pub struct CsvParseReq {
    pub input: String,
    #[serde(default)]
    pub delimiter: Option<char>, // 默认自动检测:逗号 / TAB / 分号
    #[serde(default = "default_true")]
    pub has_headers: bool,
    #[serde(default = "default_limit")]
    pub row_limit: usize,
}
fn default_true() -> bool { true }
fn default_limit() -> usize { 1000 }

#[derive(Debug, serde::Serialize)]
pub struct CsvParseResp {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total_rows: usize,        // 解析到的总行数(可能 > rows.len())
    pub column_count: usize,
    pub delimiter: char,
    pub stats: Vec<ColumnStat>,
}

#[derive(Debug, serde::Serialize)]
pub struct ColumnStat {
    pub name: String,
    pub non_empty: usize,
    pub empty: usize,
    pub unique: usize,
    pub sample_values: Vec<String>,
}

fn detect_delimiter(text: &str) -> char {
    let first_line = text.lines().next().unwrap_or("");
    let counts = [',', '\t', ';', '|'].map(|d| (d, first_line.matches(d).count()));
    counts.into_iter().max_by_key(|(_, c)| *c).map(|(d, _)| d).unwrap_or(',')
}

#[tauri::command]
pub fn csv_parse(req: CsvParseReq) -> AppResult<CsvParseResp> {
        log_and_run("csv_parse", "csv-viewer", || {

    if req.input.is_empty() {
        return Err(AppError::Invalid("输入为空".into()));
    }
    let delim = req.delimiter.unwrap_or_else(|| detect_delimiter(&req.input));
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim as u8)
        .has_headers(req.has_headers)
        .flexible(true)
        .from_reader(req.input.as_bytes());

    let headers: Vec<String> = if req.has_headers {
        rdr.headers().map_err(|e| AppError::Invalid(format!("读 header 失败: {e}")))?
            .iter().map(|s| s.to_string()).collect()
    } else {
        let n = rdr.records().next()
            .ok_or_else(|| AppError::Invalid("空输入".into()))?
            .map_err(|e| AppError::Invalid(format!("读首行失败: {e}")))?
            .len();
        (0..n).map(|i| format!("col_{i}")).collect()
    };
    let column_count = headers.len();

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut total = 0usize;
    for result in rdr.records() {
        total += 1;
        if rows.len() >= req.row_limit { continue; }
        let record = result.map_err(|e| AppError::Invalid(format!("第 {total} 行解析失败: {e}")))?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        rows.push(row);
    }

    // 列统计
    let mut stats = Vec::with_capacity(column_count);
    for col in 0..column_count {
        let mut non_empty = 0usize;
        let mut empty = 0usize;
        let mut uniques = std::collections::HashSet::new();
        let mut samples: Vec<String> = Vec::new();
        for r in &rows {
            let v = r.get(col).map(|s| s.as_str()).unwrap_or("");
            if v.is_empty() { empty += 1; } else { non_empty += 1; }
            uniques.insert(v.to_string());
            if samples.len() < 3 && !v.is_empty() && !samples.contains(&v.to_string()) {
                samples.push(v.to_string());
            }
        }
        stats.push(ColumnStat {
            name: headers.get(col).cloned().unwrap_or_default(),
            non_empty, empty,
            unique: uniques.len(),
            sample_values: samples,
        });
    }

    Ok(CsvParseResp {
        headers, rows, total_rows: total, column_count,
        delimiter: delim, stats,
    })

        })
    }

#[derive(Debug, Deserialize)]
pub struct CsvExtractReq {
    pub input: String,
    pub delimiter: Option<char>,
    pub column_index: usize,
    pub has_headers: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct CsvExtractResp {
    pub column_name: Option<String>,
    pub values: Vec<String>,
}

#[tauri::command]
pub fn csv_extract_column(req: CsvExtractReq) -> AppResult<CsvExtractResp> {
        log_and_run("csv_extract_column", "csv-viewer", || {

    let delim = req.delimiter.unwrap_or_else(|| detect_delimiter(&req.input));
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim as u8)
        .has_headers(req.has_headers)
        .from_reader(req.input.as_bytes());

    let column_name = if req.has_headers {
        let h = rdr.headers().map_err(|e| AppError::Invalid(e.to_string()))?
            .iter().nth(req.column_index).map(|s| s.to_string());
        h
    } else { None };

    let mut values = Vec::new();
    for r in rdr.records() {
        let rec = r.map_err(|e| AppError::Invalid(e.to_string()))?;
        if let Some(v) = rec.get(req.column_index) { values.push(v.to_string()); }
    }
    Ok(CsvExtractResp { column_name, values })

        })
    }