//! Release builds abort on panic with stripped symbols; keep the message and
//! location on disk so a crash report can be traced back to source.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub fn install() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(path) = log_path() {
            let _ = append(&path, &format!("{info}"));
        }
        previous(info);
    }));
}

fn append(path: &Path, message: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(
        file,
        "[{seconds}] sweeploom-gui {}: {message}",
        env!("CARGO_PKG_VERSION")
    )
}

fn log_path() -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        let home = std::env::var_os("HOME")?;
        return Some(PathBuf::from(home).join("Library/Logs/SweepLoom/panic.log"));
    }
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("XDG_STATE_HOME"))
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state"))
        })?;
    Some(base.join("SweepLoom").join("panic.log"))
}
