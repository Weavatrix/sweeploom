//! Browser screen pane state.

use std::collections::HashSet;

use sweeploom_core::SessionId;

use crate::sort::Sort;

/// Sub-view on the Browser screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserPane {
    /// OS process trees.
    Trees,
    /// Companion tabs.
    Tabs,
    /// Saved URLs.
    Later,
}

/// Selection and sort for Browser.
pub struct BrowserUi {
    /// Visible pane.
    pub pane: BrowserPane,
    /// Process table sort.
    pub tree_sort: Sort,
    /// Tab table sort.
    pub tab_sort: Sort,
    /// Stoppable trees checked for helper stop.
    pub tree_ids: HashSet<SessionId>,
    /// Row opened for member details.
    pub selected_tree: Option<SessionId>,
    /// Companion tabs checked for save/discard.
    pub tab_ids: HashSet<i64>,
    /// Unpacked Edge/Chrome extension id (`a`–`p`, 32 chars).
    pub chromium_id: String,
    /// Later URLs checked for reopen/remove.
    pub later_urls: HashSet<String>,
    /// Confirm stop helpers.
    pub confirm_helpers: bool,
    /// Confirm discard.
    pub confirm_discard: bool,
}

impl Default for BrowserUi {
    fn default() -> Self {
        Self {
            pane: BrowserPane::Trees,
            tree_sort: Sort::size_desc(),
            tab_sort: Sort::size_desc(),
            tree_ids: HashSet::new(),
            selected_tree: None,
            tab_ids: HashSet::new(),
            chromium_id: String::new(),
            later_urls: HashSet::new(),
            confirm_helpers: false,
            confirm_discard: false,
        }
    }
}
