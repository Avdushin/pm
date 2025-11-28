mod backup;
mod clipboard;
mod config;
mod crypto;
mod entry;
mod prompt;
mod session;
mod store;

use crate::backup::backup_create;
use crate::clipboard::copy_to_clipboard;
use crate::config::Config;
use crate::crypto::generate_new_config;
use crate::entry::{Entry, OtpConfig};
use crate::prompt::{prompt_password_hidden, prompt_string};
use crate::session::get_master_key_with_cache;
use crate::store::{ensure_store_dirs, list_entries, load_entry, save_entry, store_root};
use anyhow::anyhow;
use clap::{Parser, Subcommand, ValueEnum};
use time::OffsetDateTime;
use totp_rs::{Algorithm, Secret, TOTP};
use url::Url;

#[derive(Parser, Debug)]
#[command(name = "pm", version, about = "Minimal password manager in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize password store
    Init,

    /// Add a new entry
    Add {
        /// Path like work/github
        path: String,
    },

    /// Show entry
    Show {
        /// Path like work/github
        path: String,
        /// Show only password
        #[arg(long)]
        password_only: bool,
        /// Show as JSON
        #[arg(long)]
        json: bool,
    },

    /// Copy field to clipboard
    Clip {
        /// Path like work/github
        path: String,
        /// Field to copy (password by default)
        #[arg(long, value_enum)]
        field: Option<ClipField>,
    },

    /// List entries (like `pass ls`)
    ///
    /// Примеры:
    ///   pm ls
    ///   pm ls work
    Ls {
        /// Optional prefix (folder), e.g. "work" or "personal"
        prefix: Option<String>,
    },

    /// OTP management (TOTP)
    ///
    /// Примеры:
    ///   pm otp add work/github
    ///   pm otp show work/github
    ///   pm otp clip work/github
    Otp {
        #[command(subcommand)]
        cmd: OtpCommands,
    },

    /// Backup the whole store
    Backup {
        #[command(subcommand)]
        cmd: BackupCommands,
    },
}

#[derive(Subcommand, Debug)]
enum BackupCommands {
    /// Create backup archive
    ///
    /// Примеры:
    ///   pm backup create
    ///   pm backup create my_backup
    ///   pm backup create my_backup.zip
    Create {
        /// Optional backup filename
        file: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum OtpCommands {
    /// Attach OTP secret or otpauth:// URL to entry
    Add {
        /// Path like work/github
        path: String,
    },
    /// Show current OTP code
    Show {
        /// Path like work/github
        path: String,
    },
    /// Copy current OTP code to clipboard
    Clip {
        /// Path like work/github
        path: String,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ClipField {
    Password,
    Username,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => cmd_init()?,
        Commands::Add { path } => cmd_add(&path)?,
        Commands::Show {
            path,
            password_only,
            json,
        } => cmd_show(&path, password_only, json)?,
        Commands::Clip { path, field } => {
            cmd_clip(&path, field.unwrap_or(ClipField::Password))?
        }
        Commands::Ls { prefix } => cmd_ls(prefix.as_deref())?,
        Commands::Otp { cmd } => match cmd {
            OtpCommands::Add { path } => cmd_otp_add(&path)?,
            OtpCommands::Show { path } => cmd_otp_show(&path)?,
            OtpCommands::Clip { path } => cmd_otp_clip(&path)?,
        },
        Commands::Backup { cmd } => match cmd {
            BackupCommands::Create { file } => backup_create(file)?,
        },
    }

    Ok(())
}

fn cmd_init() -> anyhow::Result<()> {
    let root = store_root()?;
    if root.exists() {
        println!("Store already exists at: {}", root.display());
        return Ok(());
    }

    std::fs::create_dir_all(&root)?;
    let master_password = prompt_password_hidden("New master password: ")?;
    let confirm = prompt_password_hidden("Confirm master password: ")?;
    if master_password != confirm {
        anyhow::bail!("Passwords do not match");
    }

    let config = generate_new_config(&master_password)?;
    let config_path = crate::config::config_path()?;
    crate::config::save_config(&config, &config_path)?;

    println!("Initialized store at {}", root.display());
    Ok(())
}

fn cmd_add(path: &str) -> anyhow::Result<()> {
    ensure_store_dirs(path)?;

    let config = Config::load()?;
    let mk = get_master_key_with_cache(&config)?;

    let title = path.to_string();
    let username = prompt_string("Username (optional): ")?;
    let password = prompt_password_hidden("Password (leave empty to generate): ")?;
    let password = if password.is_empty() {
        crypto::generate_password(24, true, true, true)?
    } else {
        password
    };
    let url = prompt_string("URL (optional): ")?;
    let notes = prompt_string("Notes (optional): ")?;

    let now =
        OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339)?;

    let entry = Entry {
        version: 1,
        title,
        username: if username.is_empty() {
            None
        } else {
            Some(username)
        },
        password,
        url: if url.is_empty() { None } else { Some(url) },
        notes: if notes.is_empty() { None } else { Some(notes) },
        created_at: now.clone(),
        updated_at: now.clone(),
        otp: None,
    };

    save_entry(path, &entry, &mk)?;
    println!("Saved entry {}", path);
    Ok(())
}

fn cmd_show(path: &str, password_only: bool, json: bool) -> anyhow::Result<()> {
    let config = Config::load()?;
    let mk = get_master_key_with_cache(&config)?;

    let entry = load_entry(path, &mk)?;

    if json {
        let s = serde_json::to_string_pretty(&entry)?;
        println!("{s}");
        return Ok(());
    }

    if password_only {
        println!("{}", entry.password);
        return Ok(());
    }

    println!("Title:    {}", entry.title);
    if let Some(ref u) = entry.username {
        println!("Username: {u}");
    }
    println!("Password: {}", entry.password);
    if let Some(ref url) = entry.url {
        println!("URL:      {url}");
    }
    if let Some(ref notes) = entry.notes {
        println!("Notes:    {notes}");
    }
    println!("Created:  {}", entry.created_at);
    println!("Updated:  {}", entry.updated_at);
    if entry.otp.is_some() {
        println!("OTP:      configured");
    } else {
        println!("OTP:      not set");
    }

    Ok(())
}

fn cmd_clip(path: &str, field: ClipField) -> anyhow::Result<()> {
    let config = Config::load()?;
    let mk = get_master_key_with_cache(&config)?;

    let entry = load_entry(path, &mk)?;

    let value = match field {
        ClipField::Password => entry.password.clone(),
        ClipField::Username => entry.username.clone().unwrap_or_default(),
    };

    copy_to_clipboard(&value)?;
    println!(
        "{} copied to clipboard.",
        match field {
            ClipField::Password => "Password",
            ClipField::Username => "Username",
        }
    );

    Ok(())
}

fn cmd_ls(prefix: Option<&str>) -> anyhow::Result<()> {
    let entries = list_entries()?;

    if entries.is_empty() {
        return Ok(());
    }

    match prefix {
        None => {
            for e in entries {
                println!("{e}");
            }
        }
        Some(pref) => {
            let pref_slash = format!("{pref}/");
            for e in entries {
                if e == pref || e.starts_with(&pref_slash) {
                    println!("{e}");
                }
            }
        }
    }

    Ok(())
}

/// pm otp add PATH
fn cmd_otp_add(path: &str) -> anyhow::Result<()> {
    let config = Config::load()?;
    let mk = get_master_key_with_cache(&config)?;

    let mut entry = load_entry(path, &mk)?;
    let raw = prompt_string("OTP secret (base32) OR otpauth:// URL: ")?;
    let raw = raw.trim();

    if raw.is_empty() {
        anyhow::bail!("OTP secret cannot be empty");
    }

    let otp_cfg = parse_otp_input(raw)?;
    entry.otp = Some(otp_cfg);
    entry.updated_at =
        OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339)?;

    save_entry(path, &entry, &mk)?;
    println!("OTP configured for {}", path);
    Ok(())
}

/// pm otp show PATH
fn cmd_otp_show(path: &str) -> anyhow::Result<()> {
    let config = Config::load()?;
    let mk = get_master_key_with_cache(&config)?;
    let entry = load_entry(path, &mk)?;

    let otp_cfg = match entry.otp {
        Some(ref cfg) => cfg,
        None => {
            anyhow::bail!("No OTP configured for {}", path);
        }
    };

    let code = generate_otp_code(otp_cfg)?;
    println!("{code}");
    Ok(())
}

/// pm otp clip PATH
fn cmd_otp_clip(path: &str) -> anyhow::Result<()> {
    let config = Config::load()?;
    let mk = get_master_key_with_cache(&config)?;
    let entry = load_entry(path, &mk)?;

    let otp_cfg = match entry.otp {
        Some(ref cfg) => cfg,
        None => {
            anyhow::bail!("No OTP configured for {}", path);
        }
    };

    let code = generate_otp_code(otp_cfg)?;
    copy_to_clipboard(&code)?;
    println!("OTP code copied to clipboard.");
    Ok(())
}

/// Разобрать то, что пользователь ввёл в pm otp add:
/// - если otpauth:// URL → парсим, достаём secret/digits/period/algorithm
/// - если просто строка → считаем base32 секретом с дефолтами (totp, SHA1, 6, 30)
fn parse_otp_input(input: &str) -> anyhow::Result<OtpConfig> {
    if input.starts_with("otpauth://") {
        let url = Url::parse(input).map_err(|e| anyhow!("Invalid otpauth URL: {e}"))?;

        if url.scheme() != "otpauth" {
            return Err(anyhow!("Invalid otpauth URL scheme: {}", url.scheme()));
        }

        let kind = url.host_str().unwrap_or("").to_lowercase();
        if kind != "totp" {
            return Err(anyhow!(
                "Unsupported otpauth type '{}', only 'totp' is supported",
                kind
            ));
        }

        let mut secret: Option<String> = None;
        let mut digits: Option<u8> = None;
        let mut period: Option<u32> = None;
        let mut algo: Option<String> = None;

        for (k, v) in url.query_pairs() {
            match k.as_ref() {
                "secret" => secret = Some(v.to_string()),
                "digits" => {
                    if let Ok(d) = v.parse::<u8>() {
                        digits = Some(d);
                    }
                }
                "period" => {
                    if let Ok(p) = v.parse::<u32>() {
                        period = Some(p);
                    }
                }
                "algorithm" => {
                    algo = Some(v.to_string());
                }
                _ => {}
            }
        }

        let sec = secret.ok_or_else(|| anyhow!("otpauth URL missing 'secret' param"))?;

        // validate base32
        let _ = Secret::Encoded(sec.clone())
            .to_bytes()
            .map_err(|e| anyhow!("Invalid OTP secret (base32): {e:?}"))?;

        let algo_str = algo.unwrap_or_else(|| "SHA1".to_string()).to_uppercase();
        let digits_val = digits.unwrap_or(6);
        let period_val = period.unwrap_or(30);

        Ok(OtpConfig {
            r#type: "totp".to_string(),
            secret: sec,
            period: period_val,
            digits: digits_val,
            algo: algo_str,
        })
    } else {
        // Просто base32 секрет
        let sec = input.to_string();

        let _ = Secret::Encoded(sec.clone())
            .to_bytes()
            .map_err(|e| anyhow!("Invalid OTP secret (base32): {e:?}"))?;

        Ok(OtpConfig {
            r#type: "totp".to_string(),
            secret: sec,
            period: 30,
            digits: 6,
            algo: "SHA1".to_string(),
        })
    }
}

/// Генерирует текущий TOTP-код для данного OtpConfig
fn generate_otp_code(cfg: &OtpConfig) -> anyhow::Result<String> {
    if cfg.r#type.to_lowercase() != "totp" {
        return Err(anyhow!(
            "Unsupported OTP type '{}', only 'totp' is supported",
            cfg.r#type
        ));
    }

    if cfg.digits < 6 || cfg.digits > 8 {
        return Err(anyhow!(
            "Unsupported OTP digits '{}', expected 6–8",
            cfg.digits
        ));
    }

    let algo = match cfg.algo.to_uppercase().as_str() {
        "SHA1" => Algorithm::SHA1,
        "SHA256" => Algorithm::SHA256,
        "SHA512" => Algorithm::SHA512,
        other => {
            return Err(anyhow!(
                "Unsupported OTP algo '{}', expected SHA1/SHA256/SHA512",
                other
            ))
        }
    };

    let secret_bytes = Secret::Encoded(cfg.secret.clone())
        .to_bytes()
        .map_err(|e| anyhow!("Invalid OTP secret (base32): {e:?}"))?;

    if secret_bytes.is_empty() {
        return Err(anyhow!("OTP secret decoded to empty byte string"));
    }

    // ВАЖНО:
    // Используем *unchecked* вариант, чтобы не падать на "коротких" (80-битных) секретах
    // вроде тех, что выдает GitHub. Это нормальная практика для TOTP.
    let totp = TOTP::new_unchecked(
        algo,
        cfg.digits as usize,
        1,                  // skew
        cfg.period as u64,  // period в секундах
        secret_bytes,
    );

    let code = totp
        .generate_current()
        .map_err(|e| anyhow!("Failed to generate TOTP code: {e:?}"))?;

    Ok(code)
}
