use crate::store::store_root;
use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use time::OffsetDateTime;

/// Создать бэкап:
///   pm backup create
///   pm backup create my_backup
///   pm backup create my_backup.tar.gz
///   pm backup create my_backup.zip
pub fn backup_create(optional_path: Option<String>) -> Result<()> {
    let root = store_root()?;
    if !root.exists() {
        return Err(anyhow!(
            "Password store does not exist, run `pm init` first."
        ));
    }

    let timestamp = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)?
        .replace(':', "-");

    // Определяем имя файла
    let path = match optional_path {
        None => format!("backup_{}.zip", timestamp),
        Some(p) => {
            let p = p.trim();
            if p.is_empty() {
                format!("backup_{}.tar.gz", timestamp)
            } else if p.ends_with(".tar.gz")
                || p.ends_with(".tgz")
                || p.ends_with(".gz")
                || p.ends_with(".zip")
            {
                p.to_string()
            } else {
                // Если расширения нет — добавим .tar.gz
                format!("{p}.tar.gz")
            }
        }
    };

    // Выбор формата по расширению
    if path.ends_with(".zip") {
        backup_zip(&path, &root)?;
    } else {
        backup_tar_gz(&path, &root)?;
    }

    println!("Backup created at {}", path);
    Ok(())
}

fn backup_tar_gz(path: &str, root: &Path) -> Result<()> {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use tar::Builder;

    let file = File::create(path)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(encoder);

    // Пакуем содержимое хранилища под префиксом "pm-store"
    builder.append_dir_all("pm-store", root)?;
    let encoder = builder.into_inner()?;
    encoder.finish()?;

    Ok(())
}

fn backup_zip(path: &str, root: &Path) -> Result<()> {
    use walkdir::WalkDir;
    use zip::CompressionMethod;
    use zip::ZipWriter;
    use zip::write::FileOptions;

    let file = File::create(path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in WalkDir::new(root) {
        let entry = entry?;
        let entry_path = entry.path();
        if entry.file_type().is_file() {
            let rel_path = entry_path
                .strip_prefix(root)?
                .to_string_lossy()
                .into_owned();
            zip.start_file(rel_path, options)?;
            let mut f = File::open(entry_path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            zip.write_all(&buf)?;
        }
    }

    zip.finish()?;
    Ok(())
}
