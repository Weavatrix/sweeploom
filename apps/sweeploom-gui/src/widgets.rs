//! Small reusable egui widgets. Tokens (spacing, radii, tones) live in `theme`.

use eframe::egui::{self, Color32, CornerRadius, CursorIcon, Margin, RichText, Sense};

use crate::theme;

#[path = "widgets_cards.rs"]
mod cards;
#[path = "widgets_charts.rs"]
mod charts;
#[path = "widgets_controls.rs"]
mod controls;
#[path = "cpu_cores.rs"]
pub mod cpu_cores;
#[path = "widgets_layout.rs"]
mod layout;
#[cfg(test)]
#[path = "widgets_tests.rs"]
mod tests;

pub use crate::theme::Tone;
pub use cards::*;
pub use charts::*;
pub use controls::*;
pub use layout::*;

/// Pointing hand on anything the user can click.
pub fn pointer(response: egui::Response) -> egui::Response {
    response.on_hover_cursor(CursorIcon::PointingHand)
}

/// Standard card frame: surface, hairline, 10 px radius, 16 px padding.
#[must_use]
pub fn card_frame(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::default()
        .fill(theme::card_fill(ui))
        .stroke(theme::card_stroke(ui))
        .shadow(theme::card_shadow(ui))
        .corner_radius(CornerRadius::same(theme::RADIUS_LG))
        .inner_margin(Margin::same(theme::LG as i8))
}

/// Card that fills the available width.
pub fn card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    card_frame(ui)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui)
        })
        .inner
}

/// Grouped settings/content card with a title and optional hint.
pub fn section(
    ui: &mut egui::Ui,
    title: &str,
    hint: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    card_frame(ui).show(ui, |ui| {
        ui.set_width(ui.available_width());
        section_header(ui, title, hint);
        add_contents(ui);
    });
    ui.add_space(theme::MD);
}

/// Title + muted hint at the top of a card.
pub fn section_header(ui: &mut egui::Ui, title: &str, hint: &str) {
    ui.label(RichText::new(title).size(15.5).strong());
    if !hint.is_empty() {
        ui.add_space(2.0);
        ui.add(egui::Label::new(RichText::new(hint).size(12.5).color(theme::muted(ui))).wrap());
    }
    ui.add_space(theme::MD);
}

/// In-page section title.
pub fn section_heading(ui: &mut egui::Ui, title: &str) {
    ui.add_space(theme::XS);
    ui.label(RichText::new(title).size(15.5).strong());
    ui.add_space(theme::SM);
}

/// Screen title on its own line, then a readable subtitle.
pub fn page_title(ui: &mut egui::Ui, title: &str, hint: &str) {
    ui.label(RichText::new(title).size(22.0).strong());
    if !hint.is_empty() {
        ui.add_space(2.0);
        ui.add(egui::Label::new(RichText::new(hint).size(13.0).color(theme::muted(ui))).wrap());
    }
    ui.add_space(theme::LG);
}

/// Muted one-line caption.
pub fn caption(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    ui.add(
        egui::Label::new(
            RichText::new(text.into())
                .size(12.5)
                .color(theme::muted(ui)),
        )
        .wrap(),
    )
}

/// Status / safety chip with explicit colors. Fixed height so it centers in table rows.
pub fn chip(ui: &mut egui::Ui, text: &str, fill: Color32, color: Color32) -> egui::Response {
    badge(ui, text, fill, color, theme::RADIUS as f32)
}

fn badge(ui: &mut egui::Ui, text: &str, fill: Color32, color: Color32, radius: f32) -> egui::Response {
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        egui::FontId::proportional(12.0),
        color,
    );
    let max = (ui.available_width() - 16.0).max(24.0);
    let width = galley.size().x.min(max) + 16.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 20.0), Sense::hover());
    ui.painter().rect_filled(rect, radius, fill);
    let pos = egui::pos2(rect.left() + 8.0, rect.center().y - galley.size().y / 2.0);
    ui.painter()
        .with_clip_rect(rect.shrink2(egui::vec2(6.0, 0.0)))
        .galley(pos, galley, color);
    response
}

/// Rounded badge tinted by tone. Fixed 20 px height so it centers in any row.
pub fn pill(ui: &mut egui::Ui, text: &str, tone: Tone) -> egui::Response {
    let fill = theme::tone_bg(ui, tone);
    badge(ui, text, fill, theme::tone(ui, tone), 10.0)
}

/// Session / process status as a tinted pill.
pub fn status_pill(ui: &mut egui::Ui, label: &str) -> egui::Response {
    pill(ui, label, theme::status_tone(label))
}

/// Quiet toolbar strip behind action buttons.
pub fn toolbar(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::default()
        .fill(theme::card_fill(ui))
        .stroke(theme::card_stroke(ui))
        .corner_radius(CornerRadius::same(theme::RADIUS_LG))
        .inner_margin(Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(add_contents);
        });
    ui.add_space(theme::MD);
}

/// Skip global scan chatter; keep real action results.
pub fn action_note(ui: &mut egui::Ui, message: Option<&str>) {
    let Some(text) = message else {
        return;
    };
    if crate::format::is_scan_chatter(text) {
        return;
    }
    ui.label(text);
}

/// Compact "?" that shows `help` on hover. ASCII so it cannot tofu.
pub fn help_mark(ui: &mut egui::Ui, help: &str) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), Sense::hover());
    let color = if response.hovered() {
        theme::accent()
    } else {
        theme::muted(ui)
    };
    ui.painter()
        .circle_stroke(rect.center(), 7.5, egui::Stroke::new(1.2_f32, color));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "?",
        egui::FontId::proportional(11.0),
        color,
    );
    response.on_hover_text(help);
}

fn hover_stroke(ui: &egui::Ui, id: egui::Id, rect: egui::Rect, hovered: bool, radius: f32) {
    let t = ui.ctx().animate_bool_with_time(id, hovered, 0.12);
    if t > 0.02 {
        ui.painter().rect_stroke(
            rect,
            radius,
            egui::Stroke::new(
                1.0_f32,
                theme::lerp(Color32::TRANSPARENT, theme::accent(), t),
            ),
            egui::StrokeKind::Inside,
        );
    }
}

/// Dense single-line table row. Fits a checkbox at theme spacing.
pub const TABLE_ROW: f32 = 30.0;
/// Two-line table row (title + caption).
pub const TABLE_ROW_TALL: f32 = 46.0;

/// Recalculate saved column widths when the available table width changes.
pub fn table<'a>(ui: &'a mut egui::Ui, id: &str) -> egui_extras::TableBuilder<'a> {
    let width = ui.available_width();
    let width_id = ui.id().with(("table-width-v2", id));
    let changed = ui.data_mut(|data| {
        let previous = data.get_persisted::<f32>(width_id);
        let changed = previous.is_none_or(|previous| (previous - width).abs() > 0.5);
        data.insert_persisted(width_id, width);
        changed
    });
    let table = egui_extras::TableBuilder::new(ui).id_salt(id);
    if changed {
        table.reset();
    }
    table
}

/// Height that keeps a table scrolling inside the remaining window.
#[must_use]
pub fn table_scroll_height(ui: &egui::Ui) -> f32 {
    ui.available_height().max(180.0)
}

/// Title over a muted caption, vertically centered in a table cell.
pub fn two_line(
    ui: &mut egui::Ui,
    title: impl Into<egui::WidgetText>,
    sub: &str,
) -> egui::Response {
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(width, 36.0),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            let top = ui.add(egui::Label::new(title).truncate().selectable(false));
            ui.add(
                egui::Label::new(RichText::new(sub).size(12.0).color(theme::muted(ui)))
                    .truncate()
                    .selectable(false),
            );
            top
        },
    )
    .inner
}

/// Expand/collapse mark that cannot become a tofu square.
/// Closed points right, open points down. Returns true when clicked.
pub fn disclose(ui: &mut egui::Ui, open: bool, color: egui::Color32) -> bool {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), Sense::click());
    let t = ui.ctx().animate_bool_with_time(response.id, open, 0.12);
    let c = rect.center();
    let angle = t * std::f32::consts::FRAC_PI_2;
    let rot = |x: f32, y: f32| {
        let (s, k) = angle.sin_cos();
        egui::pos2(c.x + x * k - y * s, c.y + x * s + y * k)
    };
    let points = vec![rot(2.75, 0.0), rot(-2.25, -4.6), rot(-2.25, 4.6)];
    let color = if response.hovered() {
        theme::lerp(color, ui.visuals().text_color(), 0.4)
    } else {
        color
    };
    ui.painter().add(egui::Shape::convex_polygon(
        points,
        color,
        egui::Stroke::NONE,
    ));
    pointer(response).clicked()
}
