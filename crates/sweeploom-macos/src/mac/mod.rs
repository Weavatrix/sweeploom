//! macOS implementation. Each `unsafe` block carries its own `SAFETY:` note.

mod bookmark;
mod files;
mod home;
mod panel;
mod private_api;

pub(crate) use bookmark::{Access, create_bookmark, resolve_bookmark};
pub(crate) use files::{move_to_trash, reveal_in_finder};
pub(crate) use home::{real_home, sysctl_u32};
pub(crate) use panel::pick_folders;

use objc2::rc::Retained;
use objc2_foundation::{NSError, NSURL};
use std::path::Path;

use crate::{Error, Result};

/// Foundation error → bridge error with the localized description.
fn cocoa(error: &NSError) -> Error {
    Error::Cocoa(error.localizedDescription().to_string())
}

/// File URL for an existing path; directories get the trailing-slash form.
fn file_url(path: &Path) -> Result<Retained<NSURL>> {
    let url = if path.is_dir() {
        NSURL::from_directory_path(path)
    } else {
        NSURL::from_file_path(path)
    };
    url.ok_or_else(|| Error::InvalidPath(path.to_path_buf()))
}
