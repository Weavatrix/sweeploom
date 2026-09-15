//! Tray commands applied on the UI thread.

use std::time::{Duration, Instant};

use eframe::egui::{self, ViewportCommand};

use crate::app::SweepLoomApp;
use crate::nav::Nav;
use crate::tray::{self, TrayCommand};

impl SweepLoomApp {
    pub(crate) fn ensure_tray(&mut self) {
        if self.prefs.tray_enabled && tray::is_supported() && self.tray.is_none() {
            self.tray = tray::create();
        }
    }

    pub(crate) fn apply_tray(&mut self, ctx: &egui::Context, command: TrayCommand) {
        match command {
            TrayCommand::Show => self.leave_background(ctx),
            TrayCommand::Hide => self.enter_background(ctx),
            TrayCommand::Toggle => {
                if self.hidden {
                    self.leave_background(ctx);
                } else {
                    self.enter_background(ctx);
                }
            }
            TrayCommand::Open(nav) => {
                self.nav = nav;
                self.leave_background(ctx);
            }
            TrayCommand::Rebuild => {
                self.nav = Nav::Storage;
                self.rebuild_review();
                self.leave_background(ctx);
            }
            TrayCommand::Quit => {
                self.force_quit = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        }
    }

    pub(crate) fn handle_tray(&mut self, ctx: &egui::Context) {
        if self.tray.is_some() {
            while let Some(command) = tray::poll() {
                self.apply_tray(ctx, command);
            }
        }
        let close = ctx.input(|input| input.viewport().close_requested());
        if close && !self.force_quit && self.prefs.tray_enabled && self.tray.is_some() {
            self.enter_background(ctx);
        }
    }

    pub(crate) fn stay_background(&mut self, ctx: &egui::Context) -> bool {
        if !self.hidden {
            return false;
        }
        let minimized = ctx.input(|input| input.viewport().minimized);
        if should_wake(minimized, self.hid_at.elapsed()) {
            self.leave_background(ctx);
            return false;
        }
        if self.last_quiet.elapsed() >= Duration::from_secs(60) {
            self.sampler.pump_quiet();
            self.last_quiet = Instant::now();
        }
        ctx.request_repaint_after(if self.tray.is_some() {
            Duration::from_millis(200)
        } else {
            Duration::from_secs(60)
        });
        true
    }
}

/// Taskbar restore must wake the UI. Minimized(true) is not Visible(false).
#[must_use]
pub(crate) fn should_wake(minimized: Option<bool>, hid_for: std::time::Duration) -> bool {
    hid_for >= std::time::Duration::from_millis(250) && minimized == Some(false)
}

#[cfg(test)]
mod tests {
    use super::should_wake;
    use std::time::Duration;

    #[test]
    fn taskbar_restore_wakes_after_hide_settles() {
        assert!(!should_wake(Some(false), Duration::from_millis(50)));
        assert!(should_wake(Some(false), Duration::from_millis(250)));
        assert!(!should_wake(Some(true), Duration::from_secs(5)));
        assert!(!should_wake(None, Duration::from_secs(5)));
    }
}
