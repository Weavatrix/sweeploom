//! Header and sidebar chrome.

use eframe::egui::{self, Color32, Margin, RichText, Stroke};

use crate::app::SweepLoomApp;
use crate::icons;
use crate::nav::Nav;
use crate::screens;
use crate::theme;
use crate::widgets;

pub fn draw(ctx: &egui::Context, app: &mut SweepLoomApp) {
    egui::TopBottomPanel::top("header")
        .exact_height(56.0)
        .frame(
            egui::Frame::side_top_panel(&ctx.style())
                .inner_margin(Margin::symmetric(16, 10))
                .stroke(Stroke::NONE),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("SweepLoom").size(22.0).strong());
                ui.add_space(6.0);
                ui.label(RichText::new("Weavatrix").size(13.0).color(theme::accent()));
                if ui.available_width() > 420.0 {
                    ui.separator();
                    ui.label(
                        RichText::new("Reclaim the workstation, keep the workspace")
                            .size(14.0)
                            .color(theme::muted(ui)),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(app.prefs.theme.label())
                            .size(13.0)
                            .color(theme::muted(ui)),
                    );
                });
            });
            let bottom = ui.max_rect().bottom();
            ui.painter().hline(
                ui.max_rect().x_range(),
                bottom,
                Stroke::new(2.0_f32, theme::accent()),
            );
        });
    egui::SidePanel::left("nav")
        .resizable(false)
        .exact_width(200.0)
        .frame(
            egui::Frame::side_top_panel(&ctx.style())
                .inner_margin(Margin::symmetric(12, 10))
                .fill(ui_nav_fill(ctx)),
        )
        .show(ctx, |ui| {
            draw_nav(ui, app);
        });
    egui::CentralPanel::default()
        .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(Margin::symmetric(18, 12)))
        .show(ctx, |ui| {
            ui.set_width(ui.available_width());
            if matches!(app.nav, Nav::Sessions | Nav::Storage | Nav::Explorer) {
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

fn ui_nav_fill(ctx: &egui::Context) -> Color32 {
    let fill = ctx.style().visuals.panel_fill;
    if ctx.style().visuals.dark_mode {
        fill
    } else {
        Color32::from_rgb(248, 248, 250)
    }
}

fn draw_nav(ui: &mut egui::Ui, app: &mut SweepLoomApp) {
    let mut last_section = "";
    for nav in Nav::ALL {
        if nav.section() != last_section {
            last_section = nav.section();
            ui.add_space(10.0);
            ui.label(
                RichText::new(last_section)
                    .size(11.0)
                    .strong()
                    .color(theme::muted(ui)),
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
    let pale = if ui.visuals().dark_mode {
        Color32::from_rgb(58, 48, 36)
    } else {
        Color32::from_rgb(245, 232, 210)
    };
    let fill = theme::lerp(Color32::TRANSPARENT, pale, t);
    let icon = if selected {
        theme::accent()
    } else {
        theme::muted(ui)
    };
    let inner = egui::Frame::new()
        .fill(fill)
        .corner_radius(6)
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                icons::show(ui, nav.glyph(), 16.0, icon);
                ui.add(
                    egui::Label::new(RichText::new(nav.label()).size(15.0).strong())
                        .selectable(false),
                );
            });
        });
    let response = widgets::pointer(inner.response.interact(egui::Sense::click()));
    if selected {
        let rect = response.rect;
        ui.painter().rect_filled(
            egui::Rect::from_min_size(rect.left_top(), egui::vec2(3.0, rect.height())),
            0.0,
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
