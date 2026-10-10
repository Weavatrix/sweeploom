//! Each source owns its inventory, worker and view state for the app's lifetime.
use super::*;
use crate::scan_history::{Basis, ScanHistory, Source, now_ms};
use crossbeam_channel::TryRecvError;
use sweeploom_storage::DiskUsage;

pub(super) struct PaneState {
    pub listing: Option<Listing>,
    pub rx: Option<Receiver<ListingMsg>>,
    pub filter: String,
    pub sort: Sort,
    stale: bool,
    /// Ids sized by the current pass. Sizes carried over from the previous pass
    /// stay on screen but never reach scan history.
    fresh: std::collections::HashSet<String>,
}

impl Default for PaneState {
    fn default() -> Self {
        Self {
            listing: None,
            rx: None,
            filter: String::new(),
            sort: Sort::size_desc(),
            stale: false,
            fresh: std::collections::HashSet::new(),
        }
    }
}

impl PaneState {
    pub fn needs_scan(&self) -> bool {
        self.rx.is_none() && (self.listing.is_none() || self.stale)
    }

    pub fn start(&mut self, rx: Receiver<ListingMsg>) {
        self.rx = Some(rx);
        self.stale = false;
        self.fresh.clear();
    }

    // Completion is processed even while this pane is hidden. Failures keep any
    // available rows and require an explicit retry, rather than a refresh loop.
    fn poll(&mut self) -> bool {
        let Some(rx) = &self.rx else {
            return false;
        };
        let mut messages = Vec::new();
        loop {
            match rx.try_recv() {
                Ok(message) => messages.push(message),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if !messages
                        .iter()
                        .any(|message| matches!(message, ListingMsg::Done | ListingMsg::Failed(_)))
                    {
                        messages.push(ListingMsg::Failed(
                            "Storage scan interrupted; refresh to retry".into(),
                        ));
                    }
                    break;
                }
            }
        }
        let mut finished = None;
        for message in messages {
            match message {
                ListingMsg::Initial(mut listing) => {
                    self.carry_over(&mut listing);
                    self.listing = Some(listing);
                }
                ListingMsg::Measured(mut item) => {
                    // Status-only updates (Xcode verdicts) arrive before sizes.
                    let sized = item.bytes.is_some() || item.status.starts_with("Cannot inspect");
                    if sized {
                        self.fresh.insert(item.id.clone());
                    }
                    if let Some(listing) = &mut self.listing
                        && let Some(old) = listing.items.iter_mut().find(|old| old.id == item.id)
                    {
                        item.selected = old.selected;
                        if !sized {
                            item.bytes = old.bytes;
                            item.usage = old.usage;
                        }
                        *old = item;
                    }
                }
                ListingMsg::Done => finished = Some(true),
                ListingMsg::Failed(error) => {
                    self.listing.get_or_insert_with(Listing::default).note = error;
                    finished = Some(false);
                }
            }
        }
        if let Some(complete) = finished {
            self.rx = None;
            self.listing.get_or_insert_with(Listing::default).complete = complete;
        }
        finished == Some(true)
    }

    /// A rescan's first listing has no sizes yet: keep the user's selection and
    /// the previous sizes until each item is measured again.
    fn carry_over(&mut self, listing: &mut Listing) {
        for item in listing.items.iter().filter(|item| item.bytes.is_some()) {
            self.fresh.insert(item.id.clone());
        }
        let Some(previous) = &self.listing else {
            return;
        };
        let old: std::collections::HashMap<&str, &Item> = previous
            .items
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect();
        for item in &mut listing.items {
            let Some(old) = old.get(item.id.as_str()) else {
                continue;
            };
            item.selected = old.selected;
            if item.bytes.is_none() && !item.status.starts_with("Cannot inspect") {
                item.bytes = old.bytes;
                item.usage = old.usage;
            }
        }
    }

    fn measurements(&self) -> Vec<(PathBuf, DiskUsage, Basis)> {
        self.listing
            .iter()
            .flat_map(|listing| &listing.items)
            .filter(|item| self.fresh.contains(&item.id))
            .filter_map(|item| {
                item.bytes.map(|bytes| {
                    (
                        item.history_key(),
                        item.usage.unwrap_or(DiskUsage {
                            bytes,
                            logical_bytes: bytes,
                            ..Default::default()
                        }),
                        if item.usage.is_some() {
                            Basis::Allocated
                        } else {
                            Basis::Native
                        },
                    )
                })
            })
            .collect()
    }
}

impl NativeCleanup {
    pub(super) fn active(&self) -> &PaneState {
        &self.panes[&self.pane]
    }

    pub(super) fn active_mut(&mut self) -> &mut PaneState {
        self.panes.get_mut(&self.pane).expect("all panes exist")
    }

    pub(super) fn poll_listings(&mut self, history: &mut ScanHistory) {
        for state in self.panes.values_mut() {
            if state.poll() {
                history.record_batch(Source::Native, state.measurements(), now_ms());
            }
        }
    }

    pub(super) fn invalidate_files(&mut self) {
        for (pane, state) in &mut self.panes {
            if !matches!(pane, Pane::Docker | Pane::Ios)
                && (state.listing.is_some() || state.rx.is_some())
            {
                state.stale = true;
            }
        }
    }

    pub(super) fn invalidate(&mut self, pane: Pane) {
        self.panes.get_mut(&pane).expect("all panes exist").stale = true;
    }
}

#[cfg(test)]
#[path = "cleanup_state_tests.rs"]
mod tests;
