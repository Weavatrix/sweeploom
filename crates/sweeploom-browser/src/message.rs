//! Extension ↔ host JSON. The host never writes logs on stdout.

use serde::{Deserialize, Serialize};

use crate::action::TabAction;
use crate::tab::{CompanionTabs, TabSnapshot};

/// One tab action the host may ask the extension to perform.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabCommand {
    /// Browser tab id.
    pub tab_id: i64,
    /// Discard, bookmark-and-close, or focus. Close is never queued.
    pub action: TabAction,
    /// Installation / profile id. Empty means the legacy single-host queue.
    #[serde(default)]
    pub instance_id: String,
    /// Connection epoch. Stale epochs must not execute.
    #[serde(default)]
    pub epoch: u64,
    /// Idempotency key for this action.
    #[serde(default)]
    pub action_id: String,
    /// Unix ms after which the action expires.
    #[serde(default)]
    pub expires_unix_ms: u64,
    /// Approved display URL captured at queue time.
    #[serde(default)]
    pub expected_url: String,
}

impl TabCommand {
    /// Queue a tab action for one browser connection.
    #[must_use]
    pub fn new(tab_id: i64, action: TabAction, instance_id: impl Into<String>, epoch: u64) -> Self {
        Self {
            tab_id,
            action,
            instance_id: instance_id.into(),
            epoch,
            action_id: format!("{tab_id}-{}-{epoch}", action_name(action)),
            expires_unix_ms: 0,
            expected_url: String::new(),
        }
    }
}

fn action_name(action: TabAction) -> &'static str {
    match action {
        TabAction::Discard => "discard",
        TabAction::BookmarkAndClose => "bookmark",
        TabAction::Focus => "focus",
        TabAction::Close => "close",
        TabAction::Keep => "keep",
    }
}

/// Message from the WebExtension.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExtensionMessage {
    /// Companion handshake.
    #[serde(rename = "hello")]
    Hello {
        /// Extension version string.
        version: String,
        /// Browser installation / profile id.
        #[serde(default)]
        instance_id: String,
        /// Connection epoch.
        #[serde(default)]
        epoch: u64,
    },
    /// Full tab list. URLs must already have credentials stripped.
    #[serde(rename = "tabs")]
    Tabs {
        /// Tabs.
        tabs: Vec<TabSnapshot>,
        /// Current tab.
        #[serde(default)]
        active_tab_id: Option<i64>,
        /// Browser installation / profile id.
        #[serde(default)]
        instance_id: String,
        /// Connection epoch.
        #[serde(default)]
        epoch: u64,
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
    /// Apply queued reclaim actions. Empty queue stays an ack.
    #[serde(rename = "apply")]
    Apply {
        /// Actions the extension should perform now.
        actions: Vec<TabCommand>,
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
                ..
            } => Some(CompanionTabs {
                tabs,
                active_tab_id,
            }),
        }
    }
}
