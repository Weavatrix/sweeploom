//! Debug-build hooks for exercising the App Store edition in a sandboxed test
//! bundle without clicking. Never compiled into release builds.
//!
//! - `SWEEPLOOM_MAS_GRANTS=/a:/b` adds grants without the folder panel (the test
//!   bundle's entitlements must already reach those paths).
//! - `SWEEPLOOM_MAS_PANEL=/path` opens the grant panel there on the first frame.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use super::{FolderAccess, Grant};

impl FolderAccess {
    pub(super) fn seed_from_env(&mut self) {
        let Some(list) = std::env::var_os("SWEEPLOOM_MAS_GRANTS") else {
            return;
        };
        for path in std::env::split_paths(&list).filter(|path| path.is_dir()) {
            self.grants.retain(|grant| grant.path != path);
            self.grants.push(Grant {
                path,
                bookmark: Vec::new(),
                _access: None,
                error: None,
            });
        }
    }
}

/// Folder to open the grant panel at, once per launch.
pub(crate) fn panel_request() -> Option<PathBuf> {
    static ASKED: AtomicBool = AtomicBool::new(false);
    let path = std::env::var_os("SWEEPLOOM_MAS_PANEL")?;
    (!ASKED.swap(true, Ordering::Relaxed)).then(|| PathBuf::from(path))
}
