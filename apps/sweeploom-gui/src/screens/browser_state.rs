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
    /// Companion tabs checked for save/discard.
    pub tab_ids: HashSet<i64>,
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
            tab_ids: HashSet::new(),
            later_urls: HashSet::new(),
            confirm_helpers: false,
            confirm_discard: false,
        }
    }
}
