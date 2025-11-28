use anyhow::Result;
use std::io::{self, Write};

pub fn prompt_string(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
    Ok(s)
}

pub fn prompt_password_hidden(prompt: &str) -> Result<String> {
    let pw = rpassword::prompt_password(prompt)?;
    Ok(pw)
}
