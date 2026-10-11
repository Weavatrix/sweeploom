//! One path shape for comparisons and display on every platform.

use std::path::{Path, PathBuf};

/// `fs::canonicalize` without the Windows verbatim prefix (`\\?\C:\x` becomes
/// `C:\x`, `\\?\UNC\host\share` becomes `\\host\share`), so canonical paths
/// compare equal to paths built from `read_dir` and the environment. Falls
/// back to the input when the path cannot be resolved.
#[must_use]
pub fn canonical_path(path: &Path) -> PathBuf {
    let resolved = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    strip_verbatim(&resolved)
}

/// Remove the Windows verbatim prefix; other platforms return the path as is.
#[must_use]
pub fn strip_verbatim(path: &Path) -> PathBuf {
    if cfg!(windows) {
        let text = path.to_string_lossy();
        if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{rest}"));
        }
        if let Some(rest) = text.strip_prefix(r"\\?\") {
            return PathBuf::from(rest);
        }
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verbatim_prefixes_are_removed_on_windows_only() {
        let drive = strip_verbatim(Path::new(r"\\?\C:\Users\me"));
        let unc = strip_verbatim(Path::new(r"\\?\UNC\host\share\x"));
        if cfg!(windows) {
            assert_eq!(drive, PathBuf::from(r"C:\Users\me"));
            assert_eq!(unc, PathBuf::from(r"\\host\share\x"));
        } else {
            assert_eq!(drive, PathBuf::from(r"\\?\C:\Users\me"));
        }
        assert_eq!(strip_verbatim(Path::new("/tmp/x")), PathBuf::from("/tmp/x"));
    }
}
