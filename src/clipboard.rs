use anyhow::{Result, anyhow};
use std::process::{Command, Stdio};

#[cfg(not(target_os = "linux"))]
use copypasta::{ClipboardContext, ClipboardProvider};

/// Linux: используем wl-copy (Wayland) или xclip (X11).
#[cfg(target_os = "linux")]
pub fn copy_to_clipboard(value: &str) -> Result<()> {
    let has_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
    let has_x11 = std::env::var("DISPLAY").is_ok();

    // Если вообще нет ни X11, ни Wayland — скорее всего чистый tty
    if !has_wayland && !has_x11 {
        return Err(anyhow!(
            "No GUI clipboard detected (no DISPLAY or WAYLAND_DISPLAY). \
             You might be in a tty. Use:\n  pm show <path> --password-only | xclip -selection clipboard"
        ));
    }

    // Wayland: сначала пробуем wl-copy
    if has_wayland {
        if try_pipe_to("wl-copy", &[], value).is_ok() {
            return Ok(());
        }
    }

    // X11: пробуем xclip
    if has_x11 {
        if try_pipe_to("xclip", &["-selection", "clipboard"], value).is_ok() {
            return Ok(());
        }
    }

    Err(anyhow!(
        "Failed to copy to clipboard: wl-copy/xclip not available or failed.\n\
         Try installing `wl-clipboard` or `xclip`, or use:\n\
         pm show <path> --password-only | xclip -selection clipboard"
    ))
}

#[cfg(target_os = "linux")]
fn try_pipe_to(cmd: &str, args: &[&str], value: &str) -> Result<()> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null()) // глушим болтовню утилиты
        .spawn()
        .map_err(|e| anyhow!("failed to spawn {}: {e}", cmd))?;

    {
        use std::io::Write;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(value.as_bytes())?;
        }
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!("{} exited with status {}", cmd, status));
    }

    Ok(())
}

/// Не-Linux (Windows/macOS и прочие): используем copypasta.
#[cfg(not(target_os = "linux"))]
pub fn copy_to_clipboard(value: &str) -> Result<()> {
    let mut ctx =
        ClipboardContext::new().map_err(|e| anyhow!("Failed to initialize clipboard: {e}"))?;

    ctx.set_contents(value.to_string())
        .map_err(|e| anyhow!("Failed to copy to clipboard: {e}"))?;

    Ok(())
}
