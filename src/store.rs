use crate::crypto::{MasterKey, decrypt_entry, encrypt_entry};
use crate::entry::Entry;
use anyhow::Context;
use std::path::{Path, PathBuf};

pub fn store_root() -> anyhow::Result<PathBuf> {
    let mut dir = dirs::data_dir().ok_or_else(|| anyhow::anyhow!("cannot get data dir"))?;
    dir.push("pm-store");
    Ok(dir)
}

pub fn ensure_store_dirs(entry_path: &str) -> anyhow::Result<()> {
    let root = store_root()?;
    let entry_rel = entry_path.replace('\\', "/");
    let p = Path::new(&entry_rel);
    if let Some(parent) = p.parent() {
        let store_dir = root.join("store").join(parent);
        std::fs::create_dir_all(store_dir)?;
    } else {
        std::fs::create_dir_all(root.join("store"))?;
    }
    Ok(())
}

fn entry_file_path(entry_path: &str) -> anyhow::Result<PathBuf> {
    let root = store_root()?;
    let rel = entry_path.replace('\\', "/");
    Ok(root.join("store").join(rel).with_extension("enc"))
}

pub fn save_entry(path: &str, entry: &Entry, master_key: &MasterKey) -> anyhow::Result<()> {
    let file_path = entry_file_path(path)?;
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_vec(entry)?;
    let (nonce_b64, ct_b64) = encrypt_entry(master_key, &json)?;

    #[derive(serde::Serialize)]
    struct FileEntry<'a> {
        version: u32,
        nonce: &'a str,
        ciphertext: &'a str,
    }

    let fe = FileEntry {
        version: 1,
        nonce: &nonce_b64,
        ciphertext: &ct_b64,
    };

    let s = serde_json::to_string_pretty(&fe)?;
    std::fs::write(file_path, s)?;
    Ok(())
}

pub fn load_entry(path: &str, master_key: &MasterKey) -> anyhow::Result<Entry> {
    let file_path = entry_file_path(path)?;
    let data = std::fs::read_to_string(&file_path)
        .with_context(|| format!("cannot read entry file {}", file_path.display()))?;

    #[derive(serde::Deserialize)]
    struct FileEntry {
        version: u32,
        nonce: String,
        ciphertext: String,
    }

    let fe: FileEntry = serde_json::from_str(&data)?;
    let decrypted = decrypt_entry(master_key, &fe.nonce, &fe.ciphertext)?;
    let entry: Entry = serde_json::from_slice(&decrypted)?;
    Ok(entry)
}
