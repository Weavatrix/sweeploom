//! Trash and Finder reveal through Foundation/AppKit; no `osascript`.

use std::path::{Path, PathBuf};

use objc2::rc::Retained;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::{NSArray, NSFileManager, NSURL};

use super::{cocoa, file_url};
use crate::Result;

pub(crate) fn move_to_trash(path: &Path) -> Result<Option<PathBuf>> {
    let url = file_url(path)?;
    let mut landed: Option<Retained<NSURL>> = None;
    NSFileManager::defaultManager()
        .trashItemAtURL_resultingItemURL_error(&url, Some(&mut landed))
        .map_err(|error| cocoa(&error))?;
    Ok(landed.and_then(|url| url.to_file_path()))
}

pub(crate) fn reveal_in_finder(paths: &[PathBuf]) -> Result<()> {
    let urls = paths
        .iter()
        .map(|path| file_url(path))
        .collect::<Result<Vec<_>>>()?;
    let urls = NSArray::from_retained_slice(&urls);
    NSWorkspace::sharedWorkspace().activateFileViewerSelectingURLs(&urls);
    Ok(())
}
