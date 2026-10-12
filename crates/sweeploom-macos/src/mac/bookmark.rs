//! App-scoped security-scoped bookmarks and the start/stop access guard.

use std::path::Path;

use objc2::rc::Retained;
use objc2::runtime::Bool;
use objc2_foundation::{
    NSData, NSURL, NSURLBookmarkCreationOptions, NSURLBookmarkResolutionOptions,
};

use super::{cocoa, file_url};
use crate::{Error, Resolved, Result, ScopedAccess};

pub(crate) fn create_bookmark(path: &Path) -> Result<Vec<u8>> {
    let url = file_url(path)?;
    bookmark_for(&url)
}

/// Bookmark data for a URL the process can reach right now.
pub(super) fn bookmark_for(url: &NSURL) -> Result<Vec<u8>> {
    url.bookmarkDataWithOptions_includingResourceValuesForKeys_relativeToURL_error(
        NSURLBookmarkCreationOptions::WithSecurityScope,
        None,
        None,
    )
    .map(|data| data.to_vec())
    .map_err(|error| cocoa(&error))
}

#[allow(unsafe_code)]
pub(crate) fn resolve_bookmark(data: &[u8]) -> Result<Resolved> {
    let data = NSData::with_bytes(data);
    let mut stale = Bool::NO;
    let options = NSURLBookmarkResolutionOptions::WithSecurityScope
        | NSURLBookmarkResolutionOptions::WithoutUI;
    // SAFETY: `stale` is a valid, writable `Bool` that outlives the call; the
    // other arguments are retained Objective-C objects or plain flags.
    let url = unsafe {
        NSURL::URLByResolvingBookmarkData_options_relativeToURL_bookmarkDataIsStale_error(
            &data, options, None, &mut stale,
        )
    }
    .map_err(|error| cocoa(&error))?;
    let path = url
        .to_file_path()
        .ok_or_else(|| Error::Cocoa("bookmark does not point at a file".into()))?;
    let access = Access::start(url);
    let stale = stale.as_bool();
    // Recreate a stale bookmark while access is open, as Apple recommends.
    let refreshed = if stale {
        access.url.as_deref().and_then(|url| bookmark_for(url).ok())
    } else {
        None
    };
    Ok(Resolved {
        path,
        stale,
        refreshed,
        access: ScopedAccess(access),
    })
}

/// Balanced start/stop of a security-scoped resource.
#[derive(Debug)]
pub(crate) struct Access {
    /// Present only while access is started.
    url: Option<Retained<NSURL>>,
}

impl Access {
    #[allow(unsafe_code)]
    fn start(url: Retained<NSURL>) -> Self {
        // SAFETY: a plain message to a valid NSURL; a `true` result is balanced
        // by exactly one stop call in `Drop`.
        let started = unsafe { url.startAccessingSecurityScopedResource() };
        Self {
            url: started.then_some(url),
        }
    }

    pub(crate) fn is_active(&self) -> bool {
        self.url.is_some()
    }
}

impl Drop for Access {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        if let Some(url) = self.url.take() {
            // SAFETY: access was started on this exact URL and is stopped once.
            unsafe { url.stopAccessingSecurityScopedResource() };
        }
    }
}
