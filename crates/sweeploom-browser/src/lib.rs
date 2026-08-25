//! Native-messaging protocol types for the optional browser companion.
//!
//! The desktop app can show browser process totals without the extension.
//! Tab-level `lastAccessed` requires the companion.

#![cfg_attr(not(test), warn(missing_docs))]

mod action;
mod heat;
mod message;
mod native;
mod pressure;
mod store;
mod tab;

pub use action::{TabAction, suggested_action};
pub use heat::{TabHeat, heat_from_access};
pub use message::{ExtensionMessage, HostMessage};
pub use native::{read_frame, write_frame};
pub use pressure::{BrowserHost, BrowserPressure, family_from_name};
pub use store::{
    FRESH_MS, StoredCompanion, handle_extension_json, load_snapshot, save_snapshot, snapshot_path,
};
pub use tab::{CompanionTabs, TabSnapshot};
