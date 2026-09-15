//! Small reusable egui widgets.

use eframe::egui::{self, Color32, CornerRadius, CursorIcon, Margin, RichText, Sense};

use crate::icons::{self, Glyph};
use crate::nav::Nav;
use crate::theme;

/// How a metric should be colored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// Default ink.
    Neutral,
    /// Critical / blocked.
    Warn,
    /// Quiet / healthy.
    Ok,
}

/// One overview metric.
pub struct Metric {
    /// Stroke icon.
    pub icon: Glyph,
    /// Uppercase caption.
    pub title: String,
    /// Primary value.
    pub value: String,
    /// Supporting line.
    pub sub: String,
    /// Screen opened when the card is clicked.
    pub open: Nav,
    /// Optional 0–1 bar. Missing means “not a ratio”.
    pub fill: Option<f32>,
    /// Value coloring.
    pub tone: Tone,
}

/// Pointing hand on anything the user can click.
pub fn pointer(response: egui::Response) -> egui::Response {
    response.on_hover_cursor(CursorIcon::PointingHand)
}

/// Overview metric cards: two per row, one shared height.
pub fn metric_grid(ui: &mut egui::Ui, cards: &[Metric]) -> Option<Nav> {
    two_col_grid(ui, cards, 148.0, |ui, card, size| {
        metric_card(ui, card, size)
    })
    .map(|index| cards[index].open)
}

fn two_col_grid<T>(
    ui: &mut egui::Ui,
    items: &[T],
    height: f32,
    mut draw: impl FnMut(&mut egui::Ui, &T, egui::Vec2) -> egui::Response,
) -> Option<usize> {
    if items.is_empty() {
        return None;
    }
    let gap = 12.0;
    let cols = if ui.available_width() >= 520.0 { 2 } else { 1 };
    let width = ((ui.available_width() - gap * (cols - 1) as f32) / cols as f32).max(200.0);
    let size = egui::vec2(width, height);
    let mut clicked = None;
    for (row, chunk) in items.chunks(cols).enumerate() {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (offset, item) in chunk.iter().enumerate() {
                if draw(ui, item, size).clicked() {
                    clicked = Some(row * cols + offset);
                }
            }
        });
        ui.add_space(gap);
    }
    clicked
}

fn metric_card(ui: &mut egui::Ui, card: &Metric, size: egui::Vec2) -> egui::Response {
    let id = ui.id().with(&card.title);
    let value_color = match card.tone {
        Tone::Warn => theme::warn(),
        Tone::Ok => theme::ok(),
        Tone::Neutral => ui.visuals().text_color(),
    };
    let clicked = pointer(card_box(ui, size, 12, |ui| {
        ui.horizontal(|ui| {
            icons::show(ui, card.icon, 15.0, theme::muted(ui));
            ui.add(
                egui::Label::new(
                    RichText::new(&card.title)
                        .size(11.0)
                        .color(theme::muted(ui))
                        .strong(),
                )
                .selectable(false)
                .truncate(),
            );
        });
        ui.add_space(8.0);
        ui.add(
            egui::Label::new(
                RichText::new(&card.value)
                    .size(24.0)
                    .strong()
                    .color(value_color),
            )
            .selectable(false)
            .truncate(),
        );
        ui.add(
            egui::Label::new(RichText::new(&card.sub).size(12.5).color(theme::muted(ui)))
                .selectable(false)
                .truncate(),
        );
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            meter_sized(
                ui,
                card.fill.unwrap_or(0.0),
                if card.fill.is_some() {
                    value_color
                } else {
                    Color32::TRANSPARENT
                },
                ui.available_width().max(24.0),
            );
        });
    }));
    hover_stroke(ui, id, clicked.rect, clicked.hovered(), 12.0);
    clicked
}

fn meter_sized(ui: &mut egui::Ui, fill: f32, color: Color32, width: f32) {
    let fill = fill.clamp(0.0, 1.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width.max(8.0), 5.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, 3.0, ui.visuals().extreme_bg_color);
    if fill > 0.01 {
        let mut used = rect;
        used.set_width((rect.width() * fill).max(4.0));
        ui.painter().rect_filled(used, 3.0, color);
    }
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

/// Grouped settings/content card.
pub fn section(
    ui: &mut egui::Ui,
    title: &str,
    hint: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    egui::Frame::default()
        .fill(theme::card_fill(ui))
        .stroke(theme::card_stroke(ui))
        .shadow(theme::card_shadow(ui))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.label(RichText::new(title).size(16.0).strong());
            if !hint.is_empty() {
                ui.label(RichText::new(hint).size(13.0).color(theme::muted(ui)));
            }
            ui.add_space(8.0);
            add_contents(ui);
        });
    ui.add_space(12.0);
}

/// One opportunity / history / listing row. Display only — no click affordance.
pub fn list_row(ui: &mut egui::Ui, title: &str, meta: &str, detail: &str) {
    let _ = list_row_inner(ui, title, meta, detail, false);
}

/// Clickable listing row. Returns the row response.
pub fn list_row_at(ui: &mut egui::Ui, title: &str, meta: &str, detail: &str) -> egui::Response {
    list_row_inner(ui, title, meta, detail, true)
}

fn list_row_inner(
    ui: &mut egui::Ui,
    title: &str,
    meta: &str,
    detail: &str,
    click: bool,
) -> egui::Response {
    let id = ui.id().with(title);
    let inner = egui::Frame::default()
        .fill(theme::card_fill(ui))
        .stroke(theme::card_stroke(ui))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Label::new(RichText::new(title).strong())
                        .selectable(false)
                        .truncate(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !detail.is_empty() {
                        ui.add(
                            egui::Label::new(
                                RichText::new(detail).size(12.5).color(theme::muted(ui)),
                            )
                            .selectable(false)
                            .truncate(),
                        );
                    }
                    if !meta.is_empty() {
                        ui.add(egui::Label::new(RichText::new(meta).strong()).selectable(false));
                    }
                });
            });
        });
    let response = if click {
        pointer(inner.response.interact(Sense::click()))
    } else {
        inner.response
    };
    if click {
        hover_stroke(ui, id, response.rect, response.hovered(), 10.0);
    }
    ui.add_space(6.0);
    response
}

/// One heavy session for the Overview grid.
pub struct PressureItem {
    /// Display name.
    pub title: String,
    /// Formatted RSS.
    pub rss: String,
    /// Combined CPU percent.
    pub cpu: f32,
    /// Share of in-use RAM, 0–1.
    pub ram_share: f32,
}

/// Heavy sessions: two equal-height cards per row.
pub fn pressure_grid(ui: &mut egui::Ui, items: &[PressureItem]) -> Option<usize> {
    two_col_grid(ui, items, 84.0, pressure_card)
}

fn pressure_card(ui: &mut egui::Ui, item: &PressureItem, size: egui::Vec2) -> egui::Response {
    let id = ui.id().with(&item.title);
    let cpu_color = if item.cpu >= 80.0 {
        theme::warn()
    } else {
        theme::muted(ui)
    };
    let clicked = pointer(card_box(ui, size, 10, |ui| {
        ui.horizontal(|ui| {
            ui.add(
                egui::Label::new(RichText::new(&item.title).size(14.5).strong())
                    .selectable(false)
                    .truncate(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("{:.0}% CPU", item.cpu))
                            .size(12.5)
                            .color(cpu_color),
                    )
                    .selectable(false),
                );
                ui.add_space(10.0);
                ui.add(
                    egui::Label::new(RichText::new(&item.rss).size(14.5).strong())
                        .selectable(false),
                );
            });
        });
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("{:.0}% RAM", item.ram_share * 100.0))
                            .size(12.0)
                            .color(theme::muted(ui)),
                    )
                    .selectable(false),
                );
                meter_sized(
                    ui,
                    item.ram_share.clamp(0.0, 1.0),
                    theme::accent(),
                    (ui.available_width() - 8.0).max(48.0),
                );
            });
        });
    }));
    hover_stroke(ui, id, clicked.rect, clicked.hovered(), 10.0);
    clicked
}

fn card_box(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    radius: u8,
    add_contents: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    ui.painter().rect(
        rect,
        CornerRadius::same(radius),
        theme::card_fill(ui),
        theme::card_stroke(ui),
        egui::StrokeKind::Inside,
    );
    let inner = rect.shrink2(egui::vec2(14.0, 12.0));
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        add_contents,
    );
    response
}

/// Dense table row. Matches checkbox / disclose height after theme spacing.
pub const TABLE_ROW: f32 = 28.0;

/// Height that keeps a table scrolling inside the remaining window.
#[must_use]
pub fn table_scroll_height(ui: &egui::Ui) -> f32 {
    ui.available_height().max(180.0)
}

/// Expand/collapse mark that cannot become a tofu square.
/// Closed points right, open points down. Returns true when clicked.
pub fn disclose(ui: &mut egui::Ui, open: bool, color: egui::Color32) -> bool {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), Sense::click());
    let c = rect.center();
    let w = 4.6_f32;
    let h = 5.0_f32;
    let points = if open {
        vec![
            egui::pos2(c.x, c.y + h * 0.55),
            egui::pos2(c.x - w, c.y - h * 0.45),
            egui::pos2(c.x + w, c.y - h * 0.45),
        ]
    } else {
        vec![
            egui::pos2(c.x + h * 0.55, c.y),
            egui::pos2(c.x - h * 0.45, c.y - w),
            egui::pos2(c.x - h * 0.45, c.y + w),
        ]
    };
    ui.painter().add(egui::Shape::convex_polygon(
        points,
        color,
        egui::Stroke::NONE,
    ));
    pointer(response).clicked()
}

/// Tiny observed series. Does not invent missing samples.
pub fn sparkline(ui: &mut egui::Ui, values: &[f32], size: egui::Vec2, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if values.len() < 2 {
        return;
    }
    let min = values.iter().copied().fold(f32::MAX, f32::min);
    let max = values.iter().copied().fold(0.0_f32, f32::max);
    let span = (max - min).max(0.01);
    let last = (values.len() - 1) as f32;
    let points: Vec<egui::Pos2> = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let x = rect.left() + rect.width() * (index as f32 / last);
            let y = rect.bottom() - rect.height() * ((*value - min) / span);
            egui::Pos2::new(x, y)
        })
        .collect();
    ui.painter()
        .add(egui::Shape::line(points, egui::Stroke::new(1.4_f32, color)));
}

/// In-page section title.
pub fn section_heading(ui: &mut egui::Ui, title: &str) {
    ui.label(RichText::new(title).size(15.0).strong());
    ui.add_space(8.0);
}

/// Compact screen title with a readable subtitle.
pub fn page_title(ui: &mut egui::Ui, title: &str, hint: &str) {
    ui.label(RichText::new(title).size(22.0).strong());
    if !hint.is_empty() {
        ui.label(RichText::new(hint).size(13.0).color(theme::muted(ui)));
    }
    ui.add_space(14.0);
}

/// Status / safety chip.
pub fn chip(ui: &mut egui::Ui, text: &str, fill: Color32, color: Color32) {
    egui::Frame::new()
        .fill(fill)
        .corner_radius(6)
        .inner_margin(Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.5).color(color).strong());
        });
}

/// Quiet toolbar strip behind action buttons.
pub fn toolbar(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::default()
        .fill(theme::card_fill(ui))
        .stroke(theme::card_stroke(ui))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(12, 8))
        .show(ui, add_contents);
    ui.add_space(10.0);
}

/// Filled action button for a primary, non-destructive step.
pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    pointer(ui.add(egui::Button::new(RichText::new(text).color(Color32::WHITE)).fill(theme::ink())))
}

/// Dark action button for apply / stop.
pub fn apply_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    pointer(
        ui.add(egui::Button::new(RichText::new(text).color(Color32::WHITE)).fill(theme::warn())),
    )
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
    let mark = ui.add(
        egui::Label::new(RichText::new("?").size(13.0).color(theme::muted(ui)))
            .sense(Sense::hover())
            .selectable(false),
    );
    mark.on_hover_text(help);
}
