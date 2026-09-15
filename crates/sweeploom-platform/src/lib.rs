//! OS adapters. Core and the planner never call `cfg(target_os)` themselves.

#![cfg_attr(not(test), warn(missing_docs))]

use std::path::{Path, PathBuf};

use etcetera::BaseStrategy;
use sweeploom_core::ProcessKey;

/// Platform error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Filesystem or OS call failed.
    #[error("{0}")]
    Io(#[from] std::io::Error),
    /// Requested capability is unavailable.
    #[error("capability unavailable: {0}")]
    Capability(&'static str),
}

/// Result alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Well-known user and system locations. Discovery only — never delete from here
/// without going through `sweeploom-exec`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserLocations {
    /// Home directory.
    pub home: PathBuf,
    /// Downloads.
    pub downloads: Option<PathBuf>,
    /// User temp.
    pub temp: PathBuf,
    /// User cache (`~/Library/Caches`, `%LOCALAPPDATA%`, `XDG_CACHE_HOME`).
    pub cache: Option<PathBuf>,
    /// SweepLoom's own config directory.
    pub app_config: PathBuf,
    /// SweepLoom's own data directory.
    pub app_data: PathBuf,
}

impl UserLocations {
    /// Resolve locations for the current user.
    #[must_use]
    pub fn current() -> Self {
        let home = etcetera::home_dir().unwrap_or_else(|_| PathBuf::from("."));
        let strategy = etcetera::choose_base_strategy().ok();
        let app_config = strategy.as_ref().map_or_else(
            || home.join(".sweeploom"),
            |item| item.config_dir().join("sweeploom"),
        );
        let app_data = strategy.as_ref().map_or_else(
            || home.join(".sweeploom"),
            |item| item.data_dir().join("sweeploom"),
        );
        let cache = strategy.as_ref().map(BaseStrategy::cache_dir);
        Self {
            downloads: Some(home.join("Downloads")),
            temp: std::env::temp_dir(),
            cache: cache.or_else(|| default_user_cache(&home)),
            app_config,
            app_data,
            home,
        }
    }
}

fn default_user_cache(home: &Path) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let _ = home;
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        Some(home.join("Library").join("Caches"))
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        Some(
            std::env::var_os("XDG_CACHE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".cache")),
        )
    }
}

/// Disk space snapshot for a path's volume.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiskSpace {
    /// Total bytes.
    pub total_bytes: u64,
    /// Available bytes.
    pub available_bytes: u64,
}

impl DiskSpace {
    /// Used bytes.
    #[must_use]
    pub const fn used_bytes(self) -> u64 {
        self.total_bytes.saturating_sub(self.available_bytes)
    }
}

/// Best-effort volume space for `path`.
#[must_use]
pub fn disk_space(_path: &Path) -> Option<DiskSpace> {
    // sysinfo is used by the process crate; keep this crate free of it so core
    // platform paths can compile on the library MSRV without GUI crates.
    None
}

/// What this backend can actually do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessControlCapabilities {
    /// A supported polite stop exists (SIGTERM / Ctrl+C / app shutdown).
    pub graceful_stop: bool,
    /// Force kill is available.
    pub force_kill: bool,
}

impl ProcessControlCapabilities {
    /// Conservative default until a backend reports otherwise.
    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            graceful_stop: false,
            force_kill: false,
        }
    }

    /// Host capabilities for the current OS.
    #[must_use]
    pub const fn host() -> Self {
        Self {
            graceful_stop: !cfg!(windows),
            force_kill: true,
        }
    }
}

/// Process-control backend. Never default to force-kill.
pub trait ProcessControlBackend {
    /// What this backend supports. Unsupported graceful is not a successful Term.
    fn capabilities(&self) -> ProcessControlCapabilities {
        ProcessControlCapabilities::host()
    }
    /// Ask the process to stop politely (Ctrl+C / SIGTERM / WM_CLOSE).
    fn request_graceful_stop(&self, key: ProcessKey) -> Result<()>;
    /// Terminate after the graceful timeout.
    fn terminate(&self, key: ProcessKey) -> Result<()>;
    /// Force kill. Only after explicit escalation.
    fn force_kill(&self, key: ProcessKey) -> Result<()>;
}

/// Native file identity. Missing fields mean identity is unknown.
#[must_use]
pub fn file_identity(path: &Path) -> Option<sweeploom_core::FileIdentity> {
    let meta = std::fs::symlink_metadata(path).ok()?;
    if meta.file_type().is_symlink() {
        return None;
    }
    let (volume, file_id) = native_identity(path, &meta)?;
    Some(sweeploom_core::FileIdentity {
        volume,
        file_id,
        is_dir: meta.is_dir(),
    })
}

#[cfg(windows)]
fn native_identity(path: &Path, meta: &std::fs::Metadata) -> Option<(u64, u64)> {
    use std::os::windows::fs::MetadataExt;
    // `volume_serial_number` / `file_index` are still unstable. Creation time
    // plus a path fingerprint identifies the same directory object for plan/apply.
    let created = meta.creation_time();
    if created == 0 {
        return None;
    }
    Some((path_fingerprint(path), created))
}

#[cfg(unix)]
fn native_identity(_path: &Path, meta: &std::fs::Metadata) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    Some((meta.dev(), meta.ino()))
}

#[cfg(not(any(windows, unix)))]
fn native_identity(_path: &Path, _meta: &std::fs::Metadata) -> Option<(u64, u64)> {
    None
}

fn path_fingerprint(path: &Path) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}

/// Network capability reported to the UI. Missing data is not "zero activity".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetworkCapability {
    /// Connection table is available.
    pub connections: bool,
    /// Per-process byte counters are available.
    pub byte_rates: bool,
}

impl NetworkCapability {
    /// Conservative default: nothing claimed until a backend reports otherwise.
    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            connections: false,
            byte_rates: false,
        }
    }
}

/// Trash / recycle-bin backend. Isolated so the `trash` crate can be replaced.
pub trait TrashBackend {
    /// Move `path` to the platform trash.
    fn send_to_trash(&self, path: &Path) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_user_locations_are_absolute() {
        let locations = UserLocations::current();
        assert!(locations.home.is_absolute() || locations.home.as_os_str() == ".");
        assert!(!locations.temp.as_os_str().is_empty());
    }
}
