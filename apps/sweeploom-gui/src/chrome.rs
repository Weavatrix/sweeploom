//! Header and sidebar chrome.

use eframe::egui::{self, Color32, Margin, RichText, Stroke};

use crate::app::SweepLoomApp;
use crate::icons;
use crate::nav::Nav;
use crate::screens;
use crate::theme;
use crate::widgets;

pub fn draw(ctx: &egui::Context, app: &mut SweepLoomApp) {
    restore_if_tiny(ctx);
    egui::TopBottomPanel::top("header")
        .exact_height(56.0)
        .frame(
            egui::Frame::new()
                .fill(theme::chrome())
                .inner_margin(Margin::symmetric(16, 10))
                .stroke(Stroke::NONE),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                crate::mark::show(ui, 26.0);
                ui.add_space(8.0);
                ui.heading(
                    RichText::new("SweepLoom")
                        .size(18.0)
                        .strong()
                        .color(Color32::from_rgb(244, 246, 248)),
                );
                ui.add_space(10.0);
                widgets::chip(
                    ui,
                    "preview",
                    Color32::from_rgb(36, 32, 24),
                    theme::accent_soft(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    header_status(ui, app);
                });
            });
        });
    egui::SidePanel::left("nav")
        .resizable(false)
        .exact_width(208.0)
        .frame(
            egui::Frame::new()
                .fill(theme::chrome())
                .inner_margin(Margin::symmetric(10, 12)),
        )
        .show(ctx, |ui| {
            draw_nav(ui, app);
        });
    egui::CentralPanel::default()
        .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(Margin::symmetric(22, 16)))
        .show(ctx, |ui| {
            ui.set_width(ui.available_width());
            if matches!(
                app.nav,
                Nav::Sessions
                    | Nav::Storage
                    | Nav::Explorer
                    | Nav::History
                    | Nav::Projects
                    | Nav::Ai
            ) {
                draw_page(app, ui);
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        draw_page(app, ui);
                    });
            }
        });
}

fn header_status(ui: &mut egui::Ui, app: &SweepLoomApp) {
    let (text, fill, color) = if app.apply_rx.is_some() {
        (
            "cleanup",
            Color32::from_rgb(52, 36, 22),
            theme::accent_soft(),
        )
    } else if app.scanning {
        ("scanning", Color32::from_rgb(24, 40, 32), theme::ok())
    } else if app.observation_gap() {
        ("gap", Color32::from_rgb(48, 28, 24), theme::warn())
    } else {
        (
            app.prefs.theme.label(),
            Color32::from_rgb(32, 36, 42),
            Color32::from_rgb(168, 174, 184),
        )
    };
    widgets::chip(ui, text, fill, color);
}

fn draw_nav(ui: &mut egui::Ui, app: &mut SweepLoomApp) {
    let mut last_section = "";
    for nav in Nav::ALL {
        if nav.section() != last_section {
            last_section = nav.section();
            ui.add_space(12.0);
            ui.label(
                RichText::new(last_section)
                    .size(10.5)
                    .strong()
                    .color(Color32::from_rgb(120, 126, 138)),
            );
            ui.add_space(4.0);
        }
        nav_button(ui, app, nav);
    }
}

fn nav_button(ui: &mut egui::Ui, app: &mut SweepLoomApp, nav: Nav) {
    let selected = app.nav == nav;
    let t = ui
        .ctx()
        .animate_bool_with_time(ui.id().with(nav.label()), selected, 0.14);
    let fill = theme::lerp(Color32::TRANSPARENT, Color32::from_rgb(36, 40, 48), t);
    let icon = if selected {
        theme::accent_soft()
    } else {
        Color32::from_rgb(152, 158, 168)
    };
    let label = if selected {
        Color32::from_rgb(244, 246, 248)
    } else {
        Color32::from_rgb(188, 194, 204)
    };
    let inner = egui::Frame::new()
        .fill(fill)
        .corner_radius(8)
        .inner_margin(Margin::symmetric(10, 7))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                icons::show(ui, nav.glyph(), 15.0, icon);
                ui.add(
                    egui::Label::new(RichText::new(nav.label()).size(14.5).color(label).strong())
                        .selectable(false),
                );
            });
        });
    let response = widgets::pointer(inner.response.interact(egui::Sense::click()));
    if selected {
        let rect = response.rect;
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                rect.left_top() + egui::vec2(0.0, 6.0),
                egui::vec2(3.0, (rect.height() - 12.0).max(8.0)),
            ),
            2.0,
            theme::accent(),
        );
    }
    if response.clicked() {
        app.nav = nav;
    }
}

fn draw_page(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    match app.nav {
        Nav::Overview => screens::ui_overview(app, ui),
        Nav::Sessions => screens::ui_sessions(app, ui),
        Nav::Storage => screens::ui_review(app, ui),
        Nav::Explorer => screens::ui_storage(app, ui),
        Nav::Projects => screens::ui_projects(app, ui),
        Nav::Browser => screens::ui_browser(app, ui),
        Nav::Ai => screens::ui_ai(app, ui),
        Nav::Rules => screens::ui_rules(app, ui),
        Nav::History => screens::ui_history(app, ui),
        Nav::Settings => screens::ui_settings(app, ui),
    }
}

const MIN_W: f32 = 880.0;
const MIN_H: f32 = 600.0;

fn restore_if_tiny(ctx: &egui::Context) {
    let size = ctx.input(|input| input.viewport_rect().size());
    if !too_small(size.x, size.y) {
        return;
    }
    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1360.0, 860.0)));
}

fn too_small(width: f32, height: f32) -> bool {
    width < MIN_W || height < MIN_H
}

#[cfg(test)]
mod tests {
    #[test]
    fn persisted_icon_size_must_reset() {
        assert!(super::too_small(64.0, 64.0));
        assert!(!super::too_small(880.0, 600.0));
        assert!(!super::too_small(1360.0, 860.0));
    }
}
