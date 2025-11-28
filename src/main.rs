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
use crate::entry::Entry;
use crate::prompt::{prompt_password_hidden, prompt_string};
use crate::session::get_master_key_with_cache;
use crate::store::{ensure_store_dirs, load_entry, save_entry, store_root, list_entries};
use clap::{Parser, Subcommand, ValueEnum};
use time::OffsetDateTime;

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
        // Просто молчим — как `pass ls`, или можно:
        // println!("No entries.");
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
