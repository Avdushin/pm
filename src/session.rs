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

const SESSION_TTL_SECS: u64 = 5 * 60; // 5 минут

#[derive(Serialize, Deserialize)]
struct SessionFile {
    expires_at: u64,    // Unix time (seconds)
    master_key: String, // base64(MK)
}

fn session_path() -> Result<PathBuf> {
    // Если есть XDG_RUNTIME_DIR — кладём туда
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        Ok(PathBuf::from(dir).join("pm-session.json"))
    } else {
        // Иначе — рядом с хранилищем
        Ok(store_root()?.join("session.json"))
    }
}

fn now_unix() -> Result<u64> {
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| anyhow!("system time error: {e}"))?;
    Ok(d.as_secs())
}

#[cfg(unix)]
fn set_restrictive_permissions(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = fs::metadata(path)?;
    let mut perms = metadata.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(path, perms)
}

#[cfg(not(unix))]
fn set_restrictive_permissions(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Пытается взять master key из кэша, если он не просрочен.
/// Если кэша нет/просрочен/битый — спрашивает мастер-пароль,
/// разблокирует MK и кладёт его в кэш.
pub fn get_master_key_with_cache(config: &Config) -> Result<MasterKey> {
    // 1. Пробуем кэш
    if let Ok(path) = session_path() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(sess) = serde_json::from_str::<SessionFile>(&data) {
                let now = now_unix()?;
                if now <= sess.expires_at {
                    if let Ok(bytes) = general_purpose::STANDARD.decode(&sess.master_key) {
                        if bytes.len() == 32 {
                            let mut mk = [0u8; 32];
                            mk.copy_from_slice(&bytes);
                            return Ok(mk);
                        }
                    }
                } else {
                    // просрочен — удаляем
                    let _ = fs::remove_file(&path);
                }
            }
        }
    }

    // 2. Кэша нет или он невалиден — спрашиваем мастер-пароль
    let master_password = prompt_password_hidden("Master password: ")?;
    let mk = unlock_master_key(&master_password, config)?;

    // 3. Пишем новый кэш
    let path = session_path()?;
    let now = now_unix()?;
    let sess = SessionFile {
        expires_at: now + SESSION_TTL_SECS,
        master_key: general_purpose::STANDARD.encode(mk),
    };

    if let Ok(json) = serde_json::to_string(&sess) {
        if let Ok(()) = fs::write(&path, json) {
            let _ = set_restrictive_permissions(&path);
        }
    }

    Ok(mk)
}
