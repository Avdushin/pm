use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::store::store_root;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KdfParams {
    pub algo: String,          // "argon2id"
    pub memory_mib: u32,
    pub iterations: u32,
    pub parallelism: u32,
    pub salt: String,          // base64
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncConfig {
    pub algo: String,              // "xchacha20-poly1305"
    pub master_key_nonce: String,  // base64
    pub encrypted_master_key: String, // base64
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub version: u32,
    pub kdf: KdfParams,
    pub enc: EncConfig,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = config_path()?;
        let data = std::fs::read_to_string(&path)?;
        let cfg: Config = serde_json::from_str(&data)?;
        Ok(cfg)
    }
}

pub fn config_path() -> anyhow::Result<PathBuf> {
    // Храним config.json прямо в корне хранилища
    let root = store_root()?;
    Ok(root.join("config.json"))
}

pub fn save_config(cfg: &Config, path: &PathBuf) -> anyhow::Result<()> {
    let s = serde_json::to_string_pretty(cfg)?;
    std::fs::write(path, s)?;
    Ok(())
}
