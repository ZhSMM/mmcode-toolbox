//! ICD加解密支持
//! 
//! 主要支持两种加解密，一为pbkdf_hmac_sha256，一为AES128/CBC




pub enum IcdParam {
    PBKDF {
        text: String,  // 散列文本
        salt: String,  // 散列IV，十六进制表示
    },
}


#[derive(Debug, serde::Serialize)]
pub struct IcdResp {

}

pub fn icd_command(req: IcdParam) -> AppResult<String> {

}