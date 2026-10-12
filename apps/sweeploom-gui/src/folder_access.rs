//! App Store edition folder grants. A sandboxed app reaches a folder only after
//! the user picks it; app-scoped security bookmarks keep that grant across
//! launches. Scans, discovery and cleanup stay inside the granted folders.

use std::path::{Path, PathBuf};
use std::sync::{PoisonError, RwLock};

use crate::edition::{STORE, bridge};

#[path = "folder_access_paths.rs"]
mod paths;
pub use paths::{AI, CACHES, DEVELOPER, XCODE, grant_root, missing, short};

/// Canonical granted roots, readable from worker threads.
static ROOTS: RwLock<Vec<PathBuf>> = RwLock::new(Vec::new());

const PANEL_MESSAGE: &str = "SweepLoom can only read the folders you choose. Choose a folder (or open it and click Grant Access) to let SweepLoom measure it and clean generated files inside it.";

/// True when SweepLoom may read `path`. Always true outside the App Store edition.
#[must_use]
pub fn allowed(path: &Path) -> bool {
    !STORE || covered(&ROOTS.read().unwrap_or_else(PoisonError::into_inner), path)
}

/// Granted folders in canonical form, for discovery on worker threads.
#[must_use]
pub fn granted() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = ROOTS
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .map(|root| sweeploom_storage::canonical_path(root))
        .collect();
    roots.sort();
    roots.dedup();
    roots
}

/// Lexical, then canonical containment (`/tmp/x` under a `/private/tmp` grant).
#[must_use]
pub fn covered(roots: &[PathBuf], path: &Path) -> bool {
    if roots.iter().any(|root| path.starts_with(root)) {
        return true;
    }
    let canonical = sweeploom_storage::canonical_path(path);
    canonical != path && roots.iter().any(|root| canonical.starts_with(root))
}

/// One user grant.
pub struct Grant {
    /// Folder path.
    pub path: PathBuf,
    bookmark: Vec<u8>,
    /// Held for the app lifetime; dropping it stops access.
    _access: Option<bridge::Access>,
    /// Why access could not be restored at launch.
    pub error: Option<String>,
}

/// All grants plus their bookmark file in the app container.
#[derive(Default)]
pub struct FolderAccess {
    grants: Vec<Grant>,
    file: PathBuf,
    changed: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Stored {
    folders: Vec<StoredGrant>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StoredGrant {
    path: PathBuf,
    bookmark: String,
}

impl FolderAccess {
    /// Resolve stored bookmarks and start access. A no-op in the open-source edition.
    #[must_use]
    pub fn load(file: PathBuf) -> Self {
        let mut access = Self {
            file,
            ..Self::default()
        };
        if !STORE {
            return access;
        }
        let stored: Stored = std::fs::read_to_string(&access.file)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        let mut rewrite = false;
        for entry in stored.folders {
            let Some(bytes) = unhex(&entry.bookmark) else {
                rewrite = true;
                continue;
            };
            let grant = match bridge::resolve(&bytes) {
                Ok((path, refreshed, guard)) => {
                    rewrite |= refreshed.is_some() || path != entry.path;
                    Grant {
                        path,
                        bookmark: refreshed.unwrap_or(bytes),
                        _access: Some(guard),
                        error: None,
                    }
                }
                Err(error) => Grant {
                    path: entry.path,
                    bookmark: bytes,
                    _access: None,
                    error: Some(error),
                },
            };
            access.grants.push(grant);
        }
        if rewrite {
            access.save();
        }
        #[cfg(debug_assertions)]
        access.seed_from_env();
        access.publish();
        // Startup already rebuilds every screen; only later changes rescan.
        access.changed = false;
        access
    }

    /// Every stored grant, including ones whose access could not be restored.
    #[must_use]
    pub fn grants(&self) -> &[Grant] {
        &self.grants
    }

    /// Folders reachable right now.
    #[must_use]
    pub fn active(&self) -> Vec<PathBuf> {
        self.grants
            .iter()
            .filter(|grant| grant.error.is_none())
            .map(|grant| grant.path.clone())
            .collect()
    }

    /// Show the folder panel at `start`. Returns the folders added.
    pub fn request(&mut self, start: Option<&Path>) -> Result<Vec<PathBuf>, String> {
        let picked = bridge::pick(start, PANEL_MESSAGE)?;
        let mut added = Vec::new();
        let mut errors = Vec::new();
        for item in picked {
            match item.bookmark {
                Ok(bookmark) => {
                    self.grants.retain(|grant| grant.path != item.path);
                    // The panel's URL already grants this launch; bookmarks restore later ones.
                    self.grants.push(Grant {
                        path: item.path.clone(),
                        bookmark,
                        _access: None,
                        error: None,
                    });
                    added.push(item.path);
                }
                Err(error) => errors.push(format!("{}: {error}", item.path.display())),
            }
        }
        if !added.is_empty() {
            self.save();
            self.publish();
        }
        if errors.is_empty() {
            Ok(added)
        } else {
            Err(format!(
                "Could not remember access for {}",
                errors.join("; ")
            ))
        }
    }

    /// Forget a grant. Access stops at once for restored grants, at quit otherwise.
    pub fn remove(&mut self, path: &Path) {
        self.grants.retain(|grant| grant.path != path);
        self.save();
        self.publish();
    }

    /// True once after grants changed, so screens can rescan.
    pub fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    fn publish(&mut self) {
        let mut roots = Vec::new();
        for path in self.active() {
            let canonical = sweeploom_storage::canonical_path(&path);
            if canonical != path {
                roots.push(canonical);
            }
            roots.push(path);
        }
        *ROOTS.write().unwrap_or_else(PoisonError::into_inner) = roots;
        self.changed = true;
    }

    fn save(&self) {
        let stored = Stored {
            folders: self
                .grants
                .iter()
                .map(|grant| StoredGrant {
                    path: grant.path.clone(),
                    bookmark: hex(&grant.bookmark),
                })
                .collect(),
        };
        if let Some(parent) = self.file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string_pretty(&stored) {
            let _ = std::fs::write(&self.file, text);
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(text.get(index..index + 2)?, 16).ok())
        .collect()
}

#[cfg(debug_assertions)]
#[path = "folder_access_debug.rs"]
pub(crate) mod debug;
#[cfg(test)]
#[path = "folder_access_tests.rs"]
mod tests;
