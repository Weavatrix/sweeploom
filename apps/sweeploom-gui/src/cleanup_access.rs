//! App Store edition: list only granted locations and name the folders that
//! would reveal the rest of a source.
use super::super::{Listing, Pane};
use super::catalog::RULES;
use crate::folder_access::{allowed, grant_root};
use std::path::{Path, PathBuf};

/// Locations scanned outside the catalog rules.
const EXTRA: &[(Pane, &str)] = &[
    (Pane::Apps, "Library/Caches"),
    (Pane::Apps, "Library/Application Support/Google/Chrome"),
    (Pane::Apps, "Library/Application Support/Microsoft Edge"),
    (Pane::Apps, "Library/Application Support/BraveSoftware"),
    (Pane::Apps, "Library/Application Support/Chromium"),
    (Pane::Apps, "Library/Application Support/Arc"),
    (Pane::Apps, "Library/Application Support/Vivaldi"),
    (Pane::Data, "Downloads"),
];

/// Drop rows outside the grants; record existing folders worth granting.
pub(super) fn restrict(home: &Path, pane: Pane, listing: &mut Listing) {
    listing
        .items
        .retain(|item| item.path.as_deref().is_none_or(allowed));
    listing.access = needed(home, pane);
    if !listing.access.is_empty() {
        listing
            .note
            .push_str("\nOnly folders you granted are listed.");
    }
}

fn needed(home: &Path, pane: Pane) -> Vec<PathBuf> {
    let relatives = RULES
        .iter()
        .filter(|rule| rule.pane == pane)
        .map(|rule| rule.path)
        .chain(
            EXTRA
                .iter()
                .filter(|(extra, _)| *extra == pane)
                .map(|(_, path)| *path),
        );
    let mut roots: Vec<PathBuf> = relatives
        .filter(|relative| {
            let path = home.join(relative);
            path.exists() && !allowed(&path)
        })
        .map(|relative| grant_root(home, relative))
        .filter(|root| root.is_dir())
        .collect();
    roots.sort();
    roots.dedup();
    roots
}
