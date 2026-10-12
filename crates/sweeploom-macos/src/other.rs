//! Non-macOS stand-ins: the same API, always [`Error::Unsupported`].

use std::path::{Path, PathBuf};

use crate::{Error, FolderRequest, PickedFolder, Resolved, Result};

/// Never active off macOS.
#[derive(Debug)]
pub(crate) struct Access;

impl Access {
    pub(crate) fn is_active(&self) -> bool {
        false
    }
}

pub(crate) fn real_home() -> Option<PathBuf> {
    None
}

pub(crate) fn sysctl_u32(_name: &str) -> Option<u32> {
    None
}

pub(crate) fn pick_folders(_request: &FolderRequest) -> Result<Vec<PickedFolder>> {
    Err(Error::Unsupported)
}

pub(crate) fn create_bookmark(_path: &Path) -> Result<Vec<u8>> {
    Err(Error::Unsupported)
}

pub(crate) fn resolve_bookmark(_data: &[u8]) -> Result<Resolved> {
    Err(Error::Unsupported)
}

pub(crate) fn move_to_trash(_path: &Path) -> Result<Option<PathBuf>> {
    Err(Error::Unsupported)
}

pub(crate) fn reveal_in_finder(_paths: &[PathBuf]) -> Result<()> {
    Err(Error::Unsupported)
}
