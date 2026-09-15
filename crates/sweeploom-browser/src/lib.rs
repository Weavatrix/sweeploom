//! Native-messaging protocol types for the optional browser companion.
//!
//! The desktop app can show browser process totals without the extension.
//! Tab-level `lastAccessed` requires the companion.

#![cfg_attr(not(test), warn(missing_docs))]

mod action;
mod apply;
mod heat;
mod host;
mod host_setup;
mod install;
mod later;
mod message;
mod native;
mod pressure;
mod role;
mod store;
mod tab;

pub use action::{TabAction, suggested_action};
pub use apply::{ApplyQueue, apply_path, save_apply, take_apply, take_apply_for};
pub use heat::{TabHeat, heat_from_access};
pub use host::run_native_host;
pub use host_setup::{
    NativeHostInstall, chromium_id_path, install_native_host, load_chromium_id,
    resolve_chromium_id, save_chromium_id, sibling_host_exe, write_native_hosts,
};
pub use install::{
    FIREFOX_ADDON_ID, HOST_NAME, chromium_host_json, chromium_origin, firefox_host_json,
    is_chromium_extension_id,
};
pub use later::{
    LaterEntry, LaterShelf, add_later, later_path, load_later, open_http_urls, save_later,
};
pub use message::{ExtensionMessage, HostMessage, TabCommand};
pub use native::{read_frame, write_frame};
pub use pressure::{BrowserHost, BrowserPressure, family_from_name};
pub use role::{can_stop_helper, process_caption, process_role};
pub use store::{
    FRESH_MS, StoredCompanion, handle_extension_json, load_snapshot, save_snapshot, snapshot_path,
};
pub use tab::{CompanionTabs, TabSnapshot};
