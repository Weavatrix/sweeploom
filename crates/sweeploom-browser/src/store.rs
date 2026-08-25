//! Persist the last companion snapshot under SweepLoom app data.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::message::{ExtensionMessage, HostMessage};
use crate::tab::CompanionTabs;

/// How long a snapshot counts as a live companion.
pub const FRESH_MS: u64 = 15 * 60 * 1000;

/// Last tabs written by the native-messaging host.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredCompanion {
    /// Unix milliseconds when the host wrote the file.
    pub written_unix_ms: u64,
    /// Tabs.
    pub tabs: CompanionTabs,
}

impl StoredCompanion {
    /// True when the host wrote recently enough to treat as connected.
    #[must_use]
    pub fn is_fresh(&self, now_ms: u64) -> bool {
        now_ms.saturating_sub(self.written_unix_ms) <= FRESH_MS
    }
}

/// Path of the snapshot file.
#[must_use]
pub fn snapshot_path(app_data: &Path) -> PathBuf {
    app_data.join("companion-tabs.json")
}

/// Write tabs from the host. Creates `app_data` if needed.
pub fn save_snapshot(app_data: &Path, tabs: CompanionTabs) -> io::Result<()> {
    fs::create_dir_all(app_data)?;
    let stored = StoredCompanion {
        written_unix_ms: unix_ms(),
        tabs,
    };
    let bytes = serde_json::to_vec_pretty(&stored)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    fs::write(snapshot_path(app_data), bytes)
}

/// Load the last snapshot if the file exists.
pub fn load_snapshot(app_data: &Path) -> io::Result<Option<StoredCompanion>> {
    let path = snapshot_path(app_data);
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = fs::read(path)?;
    let stored = serde_json::from_slice(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(Some(stored))
}

/// Handle one native-messaging JSON payload. Reply is JSON for stdout.
pub fn handle_extension_json(raw: &[u8], app_data: &Path) -> Result<Vec<u8>, String> {
    let message: ExtensionMessage =
        serde_json::from_slice(raw).map_err(|error| error.to_string())?;
    let reply = match message {
        ExtensionMessage::Hello { version } => HostMessage::Ack {
            ok: true,
            detail: format!("hello {version}"),
        },
        ExtensionMessage::Tabs {
            tabs,
            active_tab_id,
        } => {
            let body = CompanionTabs {
                tabs,
                active_tab_id,
            };
            match save_snapshot(app_data, body) {
                Ok(()) => HostMessage::Ack {
                    ok: true,
                    detail: "tabs stored".into(),
                },
                Err(error) => HostMessage::Ack {
                    ok: false,
                    detail: error.to_string(),
                },
            }
        }
    };
    serde_json::to_vec(&reply).map_err(|error| error.to_string())
}

fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|item| u64::try_from(item.as_millis()).unwrap_or(0))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_does_not_need_disk() {
        let dir = std::env::temp_dir().join(format!("sweeploom-hello-{}", std::process::id()));
        let reply = handle_extension_json(br#"{"type":"hello","version":"0.1"}"#, &dir).unwrap();
        let text = String::from_utf8(reply).unwrap();
        assert!(text.contains("hello 0.1"));
        assert!(!snapshot_path(&dir).is_file());
    }

    #[test]
    fn tabs_roundtrip_on_disk() {
        let dir = std::env::temp_dir().join(format!("sweeploom-tabs-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let raw = br#"{"type":"tabs","tabs":[],"active_tab_id":null}"#;
        handle_extension_json(raw, &dir).unwrap();
        let stored = load_snapshot(&dir).unwrap().expect("file");
        assert!(stored.tabs.tabs.is_empty());
        assert!(stored.is_fresh(stored.written_unix_ms));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn stale_snapshot_is_not_fresh() {
        let stored = StoredCompanion {
            written_unix_ms: 0,
            tabs: CompanionTabs::default(),
        };
        assert!(!stored.is_fresh(FRESH_MS + 1));
    }
}
