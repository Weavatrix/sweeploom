//! Native-messaging protocol types for the optional browser companion.
//!
//! The desktop app can show browser process totals without the extension.
//! Tab-level `lastAccessed` requires the companion.

#![cfg_attr(not(test), warn(missing_docs))]

mod action;
mod heat;
mod pressure;
mod tab;

pub use action::{TabAction, suggested_action};
pub use heat::{TabHeat, heat_from_access};
pub use pressure::{BrowserHost, BrowserPressure, family_from_name};
pub use tab::{CompanionTabs, TabSnapshot};
