//! Header and sidebar chrome.

use eframe::egui::{self, Color32, Margin, RichText, Stroke};

use crate::app::SweepLoomApp;
use crate::icons;
use crate::nav::Nav;
use crate::screens;
use crate::theme;
use crate::widgets;

#[cfg(debug_assertions)]
#[path = "shots.rs"]
pub(crate) mod shots;

pub fn draw(ctx: &egui::Context, app: &mut SweepLoomApp) {
    #[cfg(debug_assertions)]
    shots::drive(ctx, app);
    if let Some(snapshot) = &app.snapshot {
        widgets::cpu_cores::record(ctx, snapshot);
    }
    restore_if_tiny(ctx);
    egui::TopBottomPanel::top("header")
        .exact_height(52.0)
        .frame(
            egui::Frame::new()
                .fill(theme::chrome())
                .inner_margin(Margin::symmetric(16, 8))
                .stroke(Stroke::NONE),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                crate::mark::show(ui, 24.0);
                ui.add_space(4.0);
                ui.heading(
                    RichText::new("SweepLoom")
                        .size(17.0)
                        .strong()
                        .color(Color32::from_rgb(244, 246, 248)),
                );
                ui.add_space(4.0);
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
        .exact_width(212.0)
        .frame(
            egui::Frame::new()
                .fill(theme::chrome())
                .inner_margin(Margin::symmetric(12, 8)),
        )
        .show(ctx, |ui| {
            draw_nav(ui, app);
        });
    egui::CentralPanel::default()
        .frame(
            egui::Frame::central_panel(&ctx.style()).inner_margin(Margin {
                left: theme::XL as i8,
                right: theme::XL as i8,
                top: 20,
                bottom: theme::LG as i8,
            }),
        )
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
                    | Nav::Cleanup
                    | Nav::DiskHistory
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
    let (text, color, busy) = if app.apply_rx.is_some() {
        ("Cleaning up", Color32::from_rgb(240, 196, 110), true)
    } else if app.scanning {
        ("Scanning", Color32::from_rgb(120, 220, 160), true)
    } else if app.observation_gap() {
        ("Observation gap", Color32::from_rgb(250, 150, 120), false)
    } else {
        ("Live", Color32::from_rgb(120, 220, 160), false)
    };
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        egui::FontId::proportional(12.5),
        color,
    );
    let size = egui::vec2(galley.size().x + 34.0, 26.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();
    painter.rect(
        rect,
        13.0,
        theme::lerp(theme::chrome(), color, 0.14),
        Stroke::new(1.0_f32, theme::lerp(theme::chrome(), color, 0.4)),
        egui::StrokeKind::Inside,
    );
    let pulse = if busy {
        let t = ui.input(|input| input.time) as f32;
        0.55 + 0.45 * (t * 3.0).sin().abs()
    } else {
        1.0
    };
    painter.circle_filled(
        egui::pos2(rect.left() + 14.0, rect.center().y),
        3.5,
        color.gamma_multiply(pulse),
    );
    painter.galley(
        egui::pos2(rect.left() + 24.0, rect.center().y - galley.size().y / 2.0),
        galley,
        color,
    );
    let tip = format!("Theme: {}", app.prefs.theme.label());
    response.on_hover_text(tip);
    if busy {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(80));
    }
}

fn draw_nav(ui: &mut egui::Ui, app: &mut SweepLoomApp) {
    let mut last_section = "";
    for nav in Nav::ALL {
        if nav.section() != last_section {
            last_section = nav.section();
            ui.add_space(theme::LG);
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(
                    RichText::new(last_section)
                        .size(10.5)
                        .strong()
                        .color(Color32::from_rgb(112, 118, 130)),
                );
            });
            ui.add_space(2.0);
        }
        nav_button(ui, app, nav);
    }
}

fn nav_button(ui: &mut egui::Ui, app: &mut SweepLoomApp, nav: Nav) {
    let selected = app.nav == nav;
    let id = ui.id().with(("nav-item", nav.label()));
    let hovered = ui.ctx().read_response(id).is_some_and(|r| r.hovered());
    let t = ui
        .ctx()
        .animate_bool_with_time(ui.id().with(nav.label()), selected, 0.14);
    let h = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), hovered, 0.1);
    let fill = theme::lerp(
        theme::lerp(Color32::TRANSPARENT, Color32::from_rgb(28, 31, 38), h),
        Color32::from_rgb(36, 40, 48),
        t,
    );
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
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                icons::show(ui, nav.glyph(), 15.0, icon);
                ui.add(
                    egui::Label::new(RichText::new(nav.label()).size(14.0).color(label).strong())
                        .selectable(false),
                );
            });
        });
    let response = widgets::pointer(ui.interact(inner.response.rect, id, egui::Sense::click()));
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
        Nav::Cleanup => crate::native_cleanup::ui(app, ui),
        Nav::Explorer => screens::ui_storage(app, ui),
        Nav::Projects => screens::ui_projects(app, ui),
        Nav::Browser => screens::ui_browser(app, ui),
        Nav::Ai => screens::ui_ai(app, ui),
        Nav::History => screens::ui_history(app, ui),
        Nav::DiskHistory => crate::disk_history::ui(app, ui),
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
