//! Login autostart. Windows uses the user Startup folder; other OS stay opt-in later.

use std::fs;
use std::path::PathBuf;

/// True when SweepLoom can register a sign-in launch.
#[must_use]
pub const fn is_supported() -> bool {
    cfg!(windows)
}

/// Whether a SweepLoom startup entry exists.
#[must_use]
pub fn is_enabled() -> bool {
    startup_path().is_some_and(|path| path.exists())
}

/// Create or remove the login launcher.
pub fn set_enabled(on: bool) -> Result<(), String> {
    if !is_supported() {
        return Err("Start with Windows is available on Windows.".to_owned());
    }
    if on { enable() } else { disable() }
}

fn startup_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let appdata = std::env::var_os("APPDATA")?;
        Some(
            PathBuf::from(appdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
                .join("Startup")
                .join("SweepLoom.cmd"),
        )
    }
    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg_attr(not(windows), allow(dead_code))]
fn enable() -> Result<(), String> {
    let path = startup_path().ok_or_else(|| "Startup folder is unavailable.".to_owned())?;
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let exe = exe.display().to_string().replace('"', "");
    let body = format!("@echo off\r\nstart \"SweepLoom\" \"{exe}\" --tray\r\n");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&path, body).map_err(|error| error.to_string())
}

#[cfg_attr(not(windows), allow(dead_code))]
fn disable() -> Result<(), String> {
    let Some(path) = startup_path() else {
        return Ok(());
    };
    if path.exists() {
        fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}
