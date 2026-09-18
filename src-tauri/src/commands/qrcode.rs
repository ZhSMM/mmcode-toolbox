//! 二维码生成(SVG)。

use qrcode::render::svg;
use qrcode::QrCode;
use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::logging::{log_and_run, log_and_run_sys};

#[derive(Debug, Deserialize)]
pub struct QrCodeReq {
    pub input: String,
    #[serde(default = "default_dark")]
    pub dark: String,
    #[serde(default = "default_light")]
    pub light: String,
}
fn default_dark() -> String { "#000000".into() }
fn default_light() -> String { "#ffffff".into() }

#[derive(Debug, serde::Serialize)]
pub struct QrCodeResp {
    pub svg: String,
    pub modules: u32,
    pub version: String,
}

#[tauri::command]
pub fn qrcode_generate(req: QrCodeReq) -> AppResult<QrCodeResp> {
        log_and_run("qrcode_generate", "qrcode", || {

    if req.input.is_empty() {
        return Err(AppError::Invalid("内容不能为空".into()));
    }
    let code = QrCode::new(req.input.as_bytes())
        .map_err(|e| AppError::Invalid(format!("QR 生成失败: {e}")))?;

    let svg_str = code.render::<svg::Color<'_>>()
        .dark_color(svg::Color(&req.dark))
        .light_color(svg::Color(&req.light))
        .build();

    Ok(QrCodeResp {
        svg: svg_str,
        modules: code.width() as u32,
        version: format!("{:?}", code.version()),
    })

        })
    }