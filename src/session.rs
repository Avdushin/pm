use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Config;
use crate::crypto::{MasterKey, unlock_master_key};
use crate::prompt::prompt_password_hidden;
use crate::store::store_root;

const SESSION_TTL_SECS: u64 = 5 * 60;

#[derive(Serialize, Deserialize)]
struct SessionFile {
    expires_at: u64,
    master_key: String,
}

pub fn session_path() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        Ok(PathBuf::from(dir).join("pm-session.json"))
    } else {
        Ok(store_root()?.join("session.json"))
    }
}

fn now_unix() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| anyhow!("system time error: {e}"))?
        .as_secs())
}

#[cfg(unix)]
fn set_perms_restrictive(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(path, perms)
}

#[cfg(not(unix))]
fn set_perms_restrictive(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

pub fn get_master_key_with_cache(cfg: &Config) -> Result<MasterKey> {
    let path = session_path()?;
    let now = now_unix()?;

    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(sess) = serde_json::from_str::<SessionFile>(&data) {
            if now <= sess.expires_at {
                let bytes = general_purpose::STANDARD.decode(&sess.master_key)?;
                if bytes.len() == 32 {
                    let mut mk = [0u8; 32];
                    mk.copy_from_slice(&bytes);
                    return Ok(mk);
                }
            } else {
                let _ = fs::remove_file(&path);
            }
        }
    }

    let master_password = prompt_password_hidden("Master password: ")?;
    let mk = unlock_master_key(&master_password, cfg)?;

    let sess = SessionFile {
        expires_at: now + SESSION_TTL_SECS,
        master_key: general_purpose::STANDARD.encode(mk),
    };

    if let Ok(json) = serde_json::to_string(&sess) {
        if fs::write(&path, json).is_ok() {
            let _ = set_perms_restrictive(&path);
        }
    }

    Ok(mk)
}
