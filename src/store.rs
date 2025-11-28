use crate::crypto::{MasterKey, decrypt_entry, encrypt_entry};
use crate::entry::Entry;
use anyhow::Context;
use std::path::{Path, PathBuf};

/// Корневая директория хранилища (например, ~/.local/share/pm-store)
pub fn store_root() -> anyhow::Result<PathBuf> {
    let mut dir = dirs::data_dir().ok_or_else(|| anyhow::anyhow!("cannot get data dir"))?;
    dir.push("pm-store");
    Ok(dir)
}

/// Убедиться, что под директорию для записи созданы все папки
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

/// Сохранить запись в зашифрованном виде
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

/// Загрузить и расшифровать запись
pub fn load_entry(path: &str, master_key: &MasterKey) -> anyhow::Result<Entry> {
    let file_path = entry_file_path(path)?;
    let data = std::fs::read_to_string(&file_path)
        .with_context(|| format!("cannot read entry file {}", file_path.display()))?;

    #[allow(dead_code)]
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

/// Вернуть список всех записей в виде путей `work/github`, `personal/mail` и т.п.
pub fn list_entries() -> anyhow::Result<Vec<String>> {
    let root = store_root()?;
    let store_dir = root.join("store");
    if !store_dir.exists() {
        return Ok(Vec::new());
    }

    fn walk(dir: &Path, root: &Path, acc: &mut Vec<String>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk(&path, root, acc)?;
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e == "enc")
                .unwrap_or(false)
            {
                let rel = path.strip_prefix(root).unwrap();
                let mut s = rel.to_string_lossy().to_string();
                if s.ends_with(".enc") {
                    s.truncate(s.len() - 4);
                }
                if std::path::MAIN_SEPARATOR != '/' {
                    s = s.replace(std::path::MAIN_SEPARATOR, "/");
                }
                acc.push(s);
            }
        }
        Ok(())
    }

    let mut entries = Vec::new();
    walk(&store_dir, &store_dir, &mut entries)?;
    entries.sort();
    Ok(entries)
}
