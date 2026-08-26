//! Local "Later" shelf. Saves title+URL without closing the tab.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

/// One saved tab. Credentials never belong here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaterEntry {
    /// Tab title at save time.
    pub title: String,
    /// http(s) URL only.
    pub url: String,
    /// Unix milliseconds when saved.
    pub saved_unix_ms: u64,
}

/// Saved tabs under SweepLoom app data.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaterShelf {
    /// Newest last.
    pub entries: Vec<LaterEntry>,
}

/// Path of the shelf file.
#[must_use]
pub fn later_path(app_data: &Path) -> PathBuf {
    app_data.join("browser-later.json")
}

/// Load the shelf. Missing file is empty.
pub fn load_later(app_data: &Path) -> io::Result<LaterShelf> {
    let path = later_path(app_data);
    if !path.is_file() {
        return Ok(LaterShelf::default());
    }
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

/// Replace the shelf on disk.
pub fn save_later(app_data: &Path, shelf: &LaterShelf) -> io::Result<()> {
    fs::create_dir_all(app_data)?;
    let bytes = serde_json::to_vec_pretty(shelf)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    fs::write(later_path(app_data), bytes)
}

/// Append http(s) tabs, skipping duplicates by URL.
pub fn add_later(shelf: &mut LaterShelf, title: &str, url: &str, now_ms: u64) -> bool {
    let Some(url) = http_url(url) else {
        return false;
    };
    if shelf.entries.iter().any(|item| item.url == url) {
        return false;
    }
    let title = if title.trim().is_empty() {
        url.clone()
    } else {
        title.trim().to_owned()
    };
    shelf.entries.push(LaterEntry {
        title,
        url,
        saved_unix_ms: now_ms,
    });
    true
}

/// Open http(s) URLs in the default browser. Returns how many launches started.
pub fn open_http_urls(urls: &[String]) -> io::Result<usize> {
    let mut n = 0_usize;
    for url in urls.iter().filter_map(|item| http_url(item)) {
        spawn_url(&url)?;
        n += 1;
    }
    Ok(n)
}

fn http_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
        Some(trimmed.to_owned())
    } else {
        None
    }
}

fn spawn_url(url: &str) -> io::Result<()> {
    drop(open_command(url).spawn()?);
    Ok(())
}

fn open_command(url: &str) -> Command {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "start", "", &format!("\"{url}\"")]);
        cmd
    }
    #[cfg(target_os = "macos")]
    {
        let mut cmd = Command::new("open");
        cmd.arg(url);
        cmd
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let mut cmd = Command::new("xdg-open");
        cmd.arg(url);
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_http_is_saved_and_deduped() {
        let mut shelf = LaterShelf::default();
        assert!(add_later(&mut shelf, "Docs", "https://example.com/a", 1));
        assert!(!add_later(&mut shelf, "Docs", "https://example.com/a", 2));
        assert!(!add_later(&mut shelf, "x", "javascript:alert(1)", 3));
        assert!(!add_later(&mut shelf, "x", "file:///tmp/x", 4));
        assert_eq!(shelf.entries.len(), 1);
    }

    #[test]
    fn roundtrip_on_disk() {
        let dir = std::env::temp_dir().join(format!("sweeploom-later-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let mut shelf = LaterShelf::default();
        assert!(add_later(&mut shelf, "A", "https://example.com", 9));
        save_later(&dir, &shelf).unwrap();
        let loaded = load_later(&dir).unwrap();
        assert_eq!(loaded, shelf);
        let _ = fs::remove_dir_all(&dir);
    }
}
