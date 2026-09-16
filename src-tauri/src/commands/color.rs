//! 颜色转换:HEX ↔ RGB ↔ HSL。

use serde::Deserialize;

use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct ColorReq {
    pub input: String,
    #[serde(default = "default_from")]
    pub from: String,
}
fn default_from() -> String { "hex".to_string() }

#[derive(Debug, serde::Serialize)]
pub struct ColorResp {
    pub hex: String,
    pub hex_long: String,
    pub rgb: (u8, u8, u8),
    pub rgba: String,
    pub hsl: (u16, u16, u16),
    pub hsla: String,
    pub luminance: f64,
    pub contrast_white: f64,
    pub contrast_black: f64,
}

fn parse_hex(s: &str) -> Result<(u8, u8, u8, u8), AppError> {
    let h = s.trim().trim_start_matches('#');
    let bytes = hex::decode(h).map_err(|e| AppError::Invalid(format!("hex 解析失败: {e}")))?;
    let (r, g, b, a) = match bytes.len() {
        3 => (bytes[0] * 17, bytes[1] * 17, bytes[2] * 17, 255u8),
        4 => (bytes[0] * 17, bytes[1] * 17, bytes[2] * 17, bytes[3] * 17),
        6 => (bytes[0], bytes[1], bytes[2], 255),
        8 => (bytes[0], bytes[1], bytes[2], bytes[3]),
        _ => return Err(AppError::Invalid(format!("hex 长度需 3/4/6/8 位,实际 {} 位", bytes.len() * 2))),
    };
    Ok((r, g, b, a))
}

fn parse_rgb(s: &str) -> Result<(u8, u8, u8, u8), AppError> {
    let inner = s.trim().trim_start_matches("rgb(").trim_start_matches("rgba(")
        .trim_end_matches(')');
    let parts: Vec<&str> = inner.split(|c: char| c == ',' || c.is_whitespace()).filter(|s| !s.is_empty()).collect();
    if parts.len() < 3 || parts.len() > 4 {
        return Err(AppError::Invalid("rgb() 需 3-4 个分量".into()));
    }
    let r = parts[0].trim().parse::<u16>().map_err(|e| AppError::Invalid(e.to_string()))?;
    let g = parts[1].trim().parse::<u16>().map_err(|e| AppError::Invalid(e.to_string()))?;
    let b = parts[2].trim().parse::<u16>().map_err(|e| AppError::Invalid(e.to_string()))?;
    let a: u8 = if parts.len() == 4 {
        let av: f64 = parts[3].trim().parse::<f64>().map_err(|e: std::num::ParseFloatError| AppError::Invalid(e.to_string()))?;
        (av.clamp(0.0, 1.0) * 255.0) as u8
    } else { 255 };
    if r > 255 || g > 255 || b > 255 {
        return Err(AppError::Invalid("rgb 分量需 0-255".into()));
    }
    Ok((r as u8, g as u8, b as u8, a))
}

fn parse_hsl(s: &str) -> Result<(u8, u8, u8, u8), AppError> {
    let inner = s.trim().trim_start_matches("hsl(").trim_start_matches("hsla(")
        .trim_end_matches(')');
    let parts: Vec<&str> = inner.split(|c: char| c == ',' || c.is_whitespace()).filter(|s| !s.is_empty()).collect();
    if parts.len() < 3 || parts.len() > 4 {
        return Err(AppError::Invalid("hsl() 需 3-4 个分量".into()));
    }
    let h: f64 = parts[0].trim().trim_end_matches("deg").parse::<f64>().map_err(|e: std::num::ParseFloatError| AppError::Invalid(e.to_string()))?;
    let s: f64 = parts[1].trim().trim_end_matches('%').parse::<f64>().map_err(|e: std::num::ParseFloatError| AppError::Invalid(e.to_string()))?;
    let l: f64 = parts[2].trim().trim_end_matches('%').parse::<f64>().map_err(|e: std::num::ParseFloatError| AppError::Invalid(e.to_string()))?;
    let a: u8 = if parts.len() == 4 {
        let av: f64 = parts[3].trim().parse::<f64>().map_err(|e: std::num::ParseFloatError| AppError::Invalid(e.to_string()))?;
        (av.clamp(0.0, 1.0) * 255.0) as u8
    } else { 255 };
    let (r, g, b) = hsl_to_rgb(h % 360.0, s.clamp(0.0, 100.0), l.clamp(0.0, 100.0));
    Ok((r, g, b, a))
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let s = s / 100.0;
    let l = l / 100.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let (r1, g1, b1) = if h_prime < 1.0 { (c, x, 0.0) }
        else if h_prime < 2.0 { (x, c, 0.0) }
        else if h_prime < 3.0 { (0.0, c, x) }
        else if h_prime < 4.0 { (0.0, x, c) }
        else if h_prime < 5.0 { (x, 0.0, c) }
        else { (c, 0.0, x) };
    let m = l - c / 2.0;
    (((r1 + m) * 255.0) as u8, ((g1 + m) * 255.0) as u8, ((b1 + m) * 255.0) as u8)
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (u16, u16, u16) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let (h, s) = if (max - min).abs() < 1e-9 {
        (0.0, 0.0)
    } else {
        let d = max - min;
        let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
        let h = if (max - r).abs() < 1e-9 {
            (g - b) / d + if g < b { 6.0 } else { 0.0 }
        } else if (max - g).abs() < 1e-9 {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
        (h * 60.0, s)
    };
    (h.round() as u16, (s * 100.0).round() as u16, (l * 100.0).round() as u16)
}

fn relative_luminance(r: u8, g: u8, b: u8) -> f64 {
    fn channel(c: u8) -> f64 {
        let c = c as f64 / 255.0;
        if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    }
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

fn contrast_ratio(l1: f64, l2: f64) -> f64 {
    let (a, b) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (a + 0.05) / (b + 0.05)
}

#[tauri::command]
pub fn color_convert(req: ColorReq) -> AppResult<ColorResp> {
    let (r, g, b, a) = match req.from.as_str() {
        "hex" => parse_hex(&req.input)?,
        "rgb" => parse_rgb(&req.input)?,
        "hsl" => parse_hsl(&req.input)?,
        other => return Err(AppError::Invalid(format!("不支持的 from: {other}"))),
    };
    let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
    let hex_long = format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a);
    let rgba = format!("rgba({},{},{},{})", r, g, b, a as f64 / 255.0);
    let (h, s, l) = rgb_to_hsl(r, g, b);
    let hsla = format!("hsla({h},{s}%,{l}%,{})", a as f64 / 255.0);
    let lum = relative_luminance(r, g, b);
    let cw = contrast_ratio(lum, 1.0);
    let cb = contrast_ratio(lum, 0.0);
    Ok(ColorResp {
        hex, hex_long, rgb: (r, g, b), rgba, hsl: (h, s, l), hsla,
        luminance: (lum * 1000.0).round() / 1000.0,
        contrast_white: (cw * 100.0).round() / 100.0,
        contrast_black: (cb * 100.0).round() / 100.0,
    })
}