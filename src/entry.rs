use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entry {
    pub version: u32,
    pub title: String,
    pub username: Option<String>,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub otp: Option<OtpConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OtpConfig {
    pub r#type: String, // "totp"
    pub secret: String,
    pub period: u32,
    pub digits: u8,
    pub algo: String, // "SHA1"
}
