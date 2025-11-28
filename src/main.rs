mod config;
mod crypto;
mod entry;
mod store;
mod clipboard;
mod prompt;
mod session;

use clap::{Parser, Subcommand, ValueEnum};
use crate::config::Config;
use crate::crypto::generate_new_config;
use crate::store::{load_entry, save_entry, ensure_store_dirs};
use crate::entry::Entry;
use crate::clipboard::copy_to_clipboard;
use crate::prompt::{prompt_password_hidden, prompt_string};
use crate::session::get_master_key_with_cache;
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
        Commands::Show { path, password_only, json } => {
            cmd_show(&path, password_only, json)?
        }
        Commands::Clip { path, field } => {
            cmd_clip(&path, field.unwrap_or(ClipField::Password))?
        }
    }

    Ok(())
}

fn cmd_init() -> anyhow::Result<()> {
    let store_root = store::store_root()?;
    if store_root.exists() {
        println!("Store already exists at: {}", store_root.display());
        return Ok(());
    }

    std::fs::create_dir_all(&store_root)?;
    let master_password = prompt_password_hidden("New master password: ")?;
    let confirm = prompt_password_hidden("Confirm master password: ")?;
    if master_password != confirm {
        anyhow::bail!("Passwords do not match");
    }

    let config = generate_new_config(&master_password)?;
    let config_path = config::config_path()?;
    config::save_config(&config, &config_path)?;

    println!("Initialized store at {}", store_root.display());
    Ok(())
}

fn cmd_add(path: &str) -> anyhow::Result<()> {
    ensure_store_dirs(path)?;

    // 1. Load config & get MK via cache
    let config = Config::load()?;
    let mk = get_master_key_with_cache(&config)?;

    // 2. Prompt entry fields
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

    let now = OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339)?;

    let entry = Entry {
        version: 1,
        title,
        username: if username.is_empty() { None } else { Some(username) },
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
