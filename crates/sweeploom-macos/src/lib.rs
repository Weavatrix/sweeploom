//! Safe macOS bridges for the sandboxed (Mac App Store) SweepLoom edition.
//!
//! The rest of the workspace forbids `unsafe`. Every Objective-C and libc call
//! the App Store edition needs lives here, behind a safe function:
//!
//! - [`real_home`]: the user's home from the password database. Inside the App
//!   Sandbox `$HOME` points at the app container instead.
//! - [`pick_folders`]: an `NSOpenPanel` for one or more folders. Picking a folder
//!   is how a sandboxed app is granted access to it.
//! - [`create_bookmark`] / [`resolve_bookmark`]: app-scoped security-scoped
//!   bookmarks, so a grant survives relaunch. [`ScopedAccess`] keeps the
//!   resource open and stops access when dropped.
//! - [`move_to_trash`] and [`reveal_in_finder`]: `NSFileManager` and
//!   `NSWorkspace`, which work for user-granted items inside the sandbox.
//!
//! Other platforms compile the same API and get [`Error::Unsupported`].

use std::fmt;
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "macos")]
use mac as imp;

#[cfg(not(target_os = "macos"))]
mod other;
#[cfg(not(target_os = "macos"))]
use other as imp;

/// Bridge failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The call exists only on macOS.
    Unsupported,
    /// AppKit panels must be shown from the main thread.
    NotMainThread,
    /// The path cannot be expressed as a file URL.
    InvalidPath(PathBuf),
    /// Foundation or AppKit reported an error (localized description).
    Cocoa(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported => f.write_str("only available on macOS"),
            Self::NotMainThread => f.write_str("folder panels must open on the main thread"),
            Self::InvalidPath(path) => write!(f, "not a file URL: {}", path.display()),
            Self::Cocoa(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for Error {}

/// Bridge result.
pub type Result<T> = std::result::Result<T, Error>;

/// What the folder panel shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FolderRequest {
    /// Explanation at the top of the panel.
    pub message: String,
    /// Confirm button title, e.g. "Grant Access". Empty keeps "Open".
    pub prompt: String,
    /// Folder the panel opens in. Confirming without a selection picks it.
    pub start: Option<PathBuf>,
    /// Show dot-folders such as `~/.cargo`.
    pub show_hidden: bool,
    /// Allow several folders at once.
    pub multiple: bool,
}

/// One folder the user picked, with its bookmark created right away.
#[derive(Debug)]
pub struct PickedFolder {
    /// File-system path of the folder.
    pub path: PathBuf,
    /// App-scoped security-scoped bookmark, or why it could not be made.
    pub bookmark: Result<Vec<u8>>,
}

/// A bookmark resolved at launch, with access already started.
#[derive(Debug)]
pub struct Resolved {
    /// Current path of the bookmarked folder (it may have moved).
    pub path: PathBuf,
    /// macOS asked for the bookmark to be recreated.
    pub stale: bool,
    /// Fresh bookmark data to store when `stale` and recreation worked.
    pub refreshed: Option<Vec<u8>>,
    /// Keeps the folder reachable until dropped.
    pub access: ScopedAccess,
}

/// An open security-scoped resource. Access stops when this is dropped.
#[derive(Debug)]
pub struct ScopedAccess(imp::Access);

impl ScopedAccess {
    /// True when macOS accepted the start-access request.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.0.is_active()
    }
}

/// The user's real home folder from the password database, not `$HOME`.
#[must_use]
pub fn real_home() -> Option<PathBuf> {
    imp::real_home()
}

/// Logical CPUs per performance level (`hw.perflevel0/1.logicalcpu`): Apple
/// silicon's performance and efficiency cores. `None` on single-level CPUs.
#[must_use]
pub fn performance_levels() -> Option<(usize, usize)> {
    let performance = imp::sysctl_u32("hw.perflevel0.logicalcpu")?;
    let efficiency = imp::sysctl_u32("hw.perflevel1.logicalcpu")?;
    (performance > 0 && efficiency > 0).then_some((performance as usize, efficiency as usize))
}

/// True when this process runs inside the App Sandbox.
#[must_use]
pub fn is_sandboxed() -> bool {
    std::env::var_os("APP_SANDBOX_CONTAINER_ID").is_some()
}

/// Show a modal folder panel. Cancel returns an empty list.
///
/// Must run on the main thread; the call blocks until the panel closes.
pub fn pick_folders(request: &FolderRequest) -> Result<Vec<PickedFolder>> {
    imp::pick_folders(request)
}

/// App-scoped security-scoped bookmark for a folder the process can reach now.
pub fn create_bookmark(path: &Path) -> Result<Vec<u8>> {
    imp::create_bookmark(path)
}

/// Resolve stored bookmark data and start accessing the folder.
pub fn resolve_bookmark(data: &[u8]) -> Result<Resolved> {
    imp::resolve_bookmark(data)
}

/// Move a file or folder to the user's Trash. Returns where it landed.
pub fn move_to_trash(path: &Path) -> Result<Option<PathBuf>> {
    imp::move_to_trash(path)
}

/// Select the given items in a Finder window.
pub fn reveal_in_finder(paths: &[PathBuf]) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }
    imp::reveal_in_finder(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_read_as_sentences() {
        assert_eq!(Error::Unsupported.to_string(), "only available on macOS");
        assert!(
            Error::InvalidPath(PathBuf::from("x"))
                .to_string()
                .ends_with('x')
        );
    }

    #[test]
    fn empty_reveal_is_a_no_op() {
        assert_eq!(reveal_in_finder(&[]), Ok(()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn real_home_is_an_existing_absolute_folder() {
        let home = real_home().expect("password database has a home");
        assert!(home.is_absolute());
        assert!(home.is_dir());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn performance_levels_match_the_logical_cpu_count() {
        if let Some((performance, efficiency)) = performance_levels() {
            let total = std::thread::available_parallelism().map_or(0, usize::from);
            assert_eq!(performance + efficiency, total);
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn bookmarks_round_trip_outside_the_sandbox() {
        let dir = std::env::temp_dir().join(format!("sweeploom-bookmark-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let data = create_bookmark(&dir).expect("bookmark");
        let resolved = resolve_bookmark(&data).expect("resolve");
        let canonical = std::fs::canonicalize(&dir).unwrap();
        assert_eq!(std::fs::canonicalize(&resolved.path).unwrap(), canonical);
        drop(resolved);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn other_platforms_report_unsupported() {
        assert_eq!(real_home(), None);
        assert!(matches!(
            pick_folders(&FolderRequest::default()),
            Err(Error::Unsupported)
        ));
        assert!(matches!(resolve_bookmark(&[1]), Err(Error::Unsupported)));
    }
}
