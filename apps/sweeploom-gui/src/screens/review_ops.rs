//! Review rebuild (off the UI thread) and size-target selection.

use super::*;

impl SweepLoomApp {
    /// Fill review from project discovery plus temp / Downloads / AI.
    ///
    /// Discovery walks generated trees and must not run on the UI thread. Rows
    /// and project sizes already shown stay until fresh values replace them.
    pub fn rebuild_review(&mut self) {
        if self.scanning {
            return;
        }
        self.project_sizes.invalidate();
        let processes = self
            .snapshot
            .as_ref()
            .map(|item| item.processes.clone())
            .unwrap_or_default();
        let (root, inventory) = self.inventory.as_ref().map_or((None, Vec::new()), |item| {
            (Some(item.root.clone()), item.projects.clone())
        });
        let current = self.current_project.as_ref().map(|item| item.0.clone());
        self.scanning = true;
        self.action_message = Some("Rebuilding review in the background…".to_owned());
        self.rebuild_rx = Some(crate::scan_job::spawn_review(
            root,
            inventory,
            current,
            processes,
            self.locations.clone(),
        ));
    }

    /// Pre-select the cheapest SAFE generated rows until `free_gb` is reached.
    pub fn select_to_free(&mut self) {
        let gb: f64 = self.free_gb.trim().parse().unwrap_or(0.0);
        let target = (gb * 1_000_000_000.0) as u64;
        if target == 0 {
            self.action_message = Some("Enter a size greater than 0 GB.".to_owned());
            return;
        }
        for row in &mut self.review {
            row.selected = false;
        }
        let mut order: Vec<usize> = self
            .review
            .iter()
            .enumerate()
            .filter(|(_, row)| can_auto_select(row))
            .map(|(index, _)| index)
            .collect();
        order.sort_by_key(|&index| {
            (
                self.review[index].candidate.rebuild.cost,
                std::cmp::Reverse(
                    self.review[index]
                        .candidate
                        .allocated_bytes
                        .unwrap_or(self.review[index].candidate.logical_bytes),
                ),
            )
        });
        let mut acc = 0_u64;
        for index in order {
            if acc >= target {
                break;
            }
            if self.review.iter().any(|row| {
                row.selected
                    && self.review[index]
                        .candidate
                        .path
                        .starts_with(&row.candidate.path)
            }) {
                continue;
            }
            self.review[index].selected = true;
            acc = acc.saturating_add(
                self.review[index]
                    .candidate
                    .allocated_bytes
                    .unwrap_or(self.review[index].candidate.logical_bytes),
            );
        }
        self.action_message = Some(format!(
            "selected {} toward {}",
            format_bytes(acc),
            format_bytes(target)
        ));
    }
}
