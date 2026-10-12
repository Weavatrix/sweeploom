//! `NSOpenPanel` folder picker. In the sandbox the panel runs out of process
//! (Powerbox) and the picked URLs carry access for this launch.

use objc2::MainThreadMarker;
use objc2_app_kit::{NSModalResponseOK, NSOpenPanel};
use objc2_foundation::{NSString, NSURL};

use super::bookmark::bookmark_for;
use crate::{Error, FolderRequest, PickedFolder, Result};

pub(crate) fn pick_folders(request: &FolderRequest) -> Result<Vec<PickedFolder>> {
    let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;
    let panel = NSOpenPanel::openPanel(mtm);
    panel.setCanChooseDirectories(true);
    panel.setCanChooseFiles(false);
    panel.setAllowsMultipleSelection(request.multiple);
    panel.setCanCreateDirectories(false);
    panel.setShowsHiddenFiles(request.show_hidden);
    if !request.message.is_empty() {
        panel.setMessage(Some(&NSString::from_str(&request.message)));
    }
    if !request.prompt.is_empty() {
        panel.setPrompt(Some(&NSString::from_str(&request.prompt)));
    }
    if let Some(url) = request
        .start
        .as_deref()
        .and_then(NSURL::from_directory_path)
    {
        panel.setDirectoryURL(Some(&url));
    }
    if panel.runModal() != NSModalResponseOK {
        return Ok(Vec::new());
    }
    Ok(panel
        .URLs()
        .to_vec()
        .into_iter()
        .filter_map(|url| {
            let path = url.to_file_path()?;
            // Bookmark now: the picked URL is what carries the sandbox grant.
            Some(PickedFolder {
                bookmark: bookmark_for(&url),
                path,
            })
        })
        .collect())
}
