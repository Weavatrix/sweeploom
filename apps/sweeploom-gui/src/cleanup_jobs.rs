use super::*;
use crate::{app::SweepLoomApp, widgets};
use eframe::egui;

impl SweepLoomApp {
    pub(super) fn refresh_native(&mut self) {
        self.refresh_native_pane(self.native_cleanup.pane);
    }
    fn refresh_native_pane(&mut self, pane: Pane) {
        let state = self
            .native_cleanup
            .panes
            .get_mut(&pane)
            .expect("all panes exist");
        if state.rx.is_some() {
            return;
        }
        let home = self.locations.home.clone();
        let (tx, rx) = crossbeam_channel::unbounded();
        state.start(rx);
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match pane {
                Pane::Docker | Pane::Ios => {
                    let listing = if pane == Pane::Docker {
                        docker_listing(&home)
                    } else {
                        ios_listing()
                    };
                    let _ = tx.send(ListingMsg::Initial(listing));
                    let _ = tx.send(ListingMsg::Done);
                }
                _ => sources::stream(&home, pane, &tx),
            }));
            if result.is_err() {
                let _ = tx.send(ListingMsg::Failed(
                    "Storage inventory failed; refresh to retry".into(),
                ));
            }
        });
    }
    pub(crate) fn poll_native(&mut self) {
        let file_busy = self.apply_rx.is_some() || self.disk_actions.busy();
        if self.native_cleanup.file_busy && !file_busy {
            self.native_cleanup.invalidate_files();
        }
        self.native_cleanup.file_busy = file_busy;
        self.native_cleanup.poll_listings(&mut self.scan_history);
        if let Some((pane, rx)) = &self.native_cleanup.applying
            && let Ok(result) = rx.try_recv()
        {
            let pane = *pane;
            self.native_cleanup.applying = None;
            self.action_message = Some(result);
            self.native_cleanup.invalidate(pane);
            self.refresh_native_pane(pane);
        }
    }
    pub(crate) fn native_dialog(&mut self, ctx: &egui::Context) {
        let Some((_, items)) = &self.native_cleanup.pending else {
            return;
        };
        let mut confirm = false;
        let mut cancel = false;
        let response = egui::Modal::new(egui::Id::new("native-cleanup-confirmation")).show(ctx, |ui| {
            ui.set_width(600.0); ui.heading("Confirm native cleanup");
            ui.label("Permanently remove only these objects through their native tool. Simulator/volume data and custom model definitions may be lost. Running objects and model versions are checked again before deletion.");
            egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| { for item in items { ui.label(format!("{} · {} · {}", item.name, item.id, item.status)); } });
            ui.horizontal(|ui| { cancel = ui.button("Cancel").clicked(); confirm = widgets::apply_button(ui, "Confirm deletion").clicked(); });
        });
        cancel |= response.should_close();
        if cancel {
            self.native_cleanup.pending = None;
        }
        if confirm && let Some((pane, items)) = self.native_cleanup.pending.take() {
            let processes = self
                .snapshot
                .as_ref()
                .map(|snapshot| snapshot.processes.clone())
                .unwrap_or_default();
            let (tx, rx) = crossbeam_channel::bounded(1);
            self.native_cleanup.applying = Some((pane, rx));
            self.action_message = Some("Applying native cleanup…".into());
            std::thread::spawn(move || {
                let result = apply_native(&items, &processes);
                let _ = tx.send(result);
            });
        }
    }
}
