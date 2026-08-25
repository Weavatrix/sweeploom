//! Extension ↔ host JSON. The host never writes logs on stdout.

use serde::{Deserialize, Serialize};

use crate::tab::{CompanionTabs, TabSnapshot};

/// Message from the WebExtension.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExtensionMessage {
    /// Companion handshake.
    #[serde(rename = "hello")]
    Hello {
        /// Extension version string.
        version: String,
    },
    /// Full tab list. URLs must already have credentials stripped.
    #[serde(rename = "tabs")]
    Tabs {
        /// Tabs.
        tabs: Vec<TabSnapshot>,
        /// Current tab.
        #[serde(default)]
        active_tab_id: Option<i64>,
    },
}

/// Message back to the extension. Never a destructive command by default.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HostMessage {
    /// Handshake or persist result.
    #[serde(rename = "ack")]
    Ack {
        /// True when the host accepted the payload.
        ok: bool,
        /// Human detail for the extension console.
        detail: String,
    },
}

impl ExtensionMessage {
    /// Tabs body when this is a tab snapshot.
    #[must_use]
    pub fn tabs(self) -> Option<CompanionTabs> {
        match self {
            Self::Hello { .. } => None,
            Self::Tabs {
                tabs,
                active_tab_id,
            } => Some(CompanionTabs {
                tabs,
                active_tab_id,
            }),
        }
    }
}
