//! Buttons, checkboxes, segmented controls, tabs and grouped inputs.

use eframe::egui::{self, Color32, Margin, RichText, Sense, Stroke, StrokeKind};

use super::pointer;
use crate::theme;

/// Filled accent button for the main, non-destructive step.
pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    pointer(
        ui.add(
            egui::Button::new(RichText::new(text).color(theme::ink()).strong())
                .fill(theme::accent())
                .stroke(Stroke::NONE),
        ),
    )
}

/// Filled destructive button for apply / stop.
pub fn apply_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    pointer(
        ui.add(
            egui::Button::new(RichText::new(text).color(Color32::WHITE).strong())
                .fill(theme::warn())
                .stroke(Stroke::NONE),
        ),
    )
}

/// Outlined destructive button. Disabled state stays clearly faded.
pub fn danger_button(ui: &mut egui::Ui, text: &str, enabled: bool) -> egui::Response {
    let color = theme::tone(ui, theme::Tone::Warn);
    pointer(
        ui.add_enabled(
            enabled,
            egui::Button::new(RichText::new(text).color(color))
                .fill(theme::tone_bg(ui, theme::Tone::Warn))
                .stroke(Stroke::new(1.0_f32, color.gamma_multiply(0.6))),
        ),
    )
}

/// Secondary button with an explicit enabled flag.
pub fn button(ui: &mut egui::Ui, text: &str, enabled: bool) -> egui::Response {
    let response = ui.add_enabled(enabled, egui::Button::new(text));
    if enabled { pointer(response) } else { response }
}

/// Square checkbox: accent fill when on, strong border when off.
pub fn check(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    check_enabled(ui, on, true)
}

/// Checkbox that can be disabled (system-critical rows).
pub fn check_enabled(ui: &mut egui::Ui, on: &mut bool, enabled: bool) -> egui::Response {
    let size = egui::vec2(18.0, ui.spacing().interact_size.y.min(24.0));
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, mut response) = ui.allocate_exact_size(size, sense);
    if enabled && response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let checked = *on;
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, enabled, checked, "")
    });
    paint_check(ui, &response, rect, checked, enabled);
    if enabled { pointer(response) } else { response }
}

fn paint_check(
    ui: &egui::Ui,
    response: &egui::Response,
    rect: egui::Rect,
    checked: bool,
    enabled: bool,
) {
    let t = ui.ctx().animate_bool_with_time(response.id, checked, 0.1);
    let alpha = if enabled { 1.0 } else { 0.35 };
    let border = if enabled && response.hovered() {
        theme::accent()
    } else {
        theme::border_strong(ui)
    };
    let fill = theme::lerp(theme::card_fill(ui), theme::accent(), t);
    let stroke = theme::lerp(border, theme::accent(), t);
    let boxed = egui::Rect::from_center_size(rect.center(), egui::vec2(16.0, 16.0));
    ui.painter().rect(
        boxed,
        theme::RADIUS_SM,
        fill.gamma_multiply(alpha),
        Stroke::new(1.5_f32, stroke.gamma_multiply(alpha)),
        StrokeKind::Inside,
    );
    if t > 0.4 {
        let c = boxed.center();
        ui.painter().add(egui::Shape::line(
            vec![
                egui::pos2(c.x - 3.8, c.y + 0.2),
                egui::pos2(c.x - 1.0, c.y + 3.0),
                egui::pos2(c.x + 4.0, c.y - 3.0),
            ],
            Stroke::new(2.0_f32, theme::ink().gamma_multiply(alpha * t)),
        ));
    }
}

/// Checkbox followed by a clickable label.
pub fn check_label(ui: &mut egui::Ui, on: &mut bool, text: &str) -> egui::Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let mut response = check(ui, on);
        let label = pointer(ui.add(egui::Label::new(text).sense(Sense::click())));
        if label.clicked() {
            *on = !*on;
            response.mark_changed();
        }
        response
    })
    .inner
}

/// macOS-style segmented control. Returns true when the value changed.
pub fn segmented<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    value: &mut T,
    options: &[(T, &str)],
) -> bool {
    let mut changed = false;
    egui::Frame::new()
        .fill(theme::inset(ui))
        .corner_radius(theme::RADIUS + 2)
        .inner_margin(Margin::same(2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (option, label) in options {
                    let selected = *value == *option;
                    if segment(ui, label, selected).clicked() && !selected {
                        *value = *option;
                        changed = true;
                    }
                }
            });
        });
    changed
}

fn segment(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    let color = if selected {
        ui.visuals().text_color()
    } else {
        theme::muted(ui)
    };
    let galley = egui::WidgetText::from(RichText::new(label).size(13.5).color(color)).into_galley(
        ui,
        Some(egui::TextWrapMode::Extend),
        f32::INFINITY,
        egui::TextStyle::Button,
    );
    let size = egui::vec2(galley.size().x + 22.0, 24.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    if selected {
        let fill = if ui.visuals().dark_mode {
            Color32::from_rgb(58, 66, 80)
        } else {
            Color32::WHITE
        };
        painter.rect(
            rect,
            theme::RADIUS,
            fill,
            theme::card_stroke(ui),
            StrokeKind::Inside,
        );
    } else if response.hovered() {
        painter.rect_filled(
            rect,
            theme::RADIUS,
            ui.visuals().widgets.hovered.weak_bg_fill,
        );
    }
    let pos = rect.center() - galley.size() / 2.0;
    painter.galley(pos, galley, color);
    pointer(response)
}

/// Underlined page tabs. Labels may carry a count, e.g. "iOS Simulator · 38 GB".
pub fn tabs<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    value: &mut T,
    options: &[(T, String)],
) -> bool {
    let mut changed = false;
    let top = ui.cursor().top();
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = theme::LG;
        for (option, label) in options {
            let selected = *value == *option;
            let color = if selected {
                ui.visuals().text_color()
            } else {
                theme::muted(ui)
            };
            let mut text = RichText::new(label).size(14.0).color(color);
            if selected {
                text = text.strong();
            }
            let response = pointer(
                ui.add(
                    egui::Label::new(text)
                        .selectable(false)
                        .sense(Sense::click()),
                ),
            );
            let line = egui::Rect::from_min_max(
                egui::pos2(response.rect.left(), response.rect.bottom() + 5.0),
                egui::pos2(response.rect.right(), response.rect.bottom() + 7.0),
            );
            if selected {
                ui.painter().rect_filled(line, 1.0, theme::accent());
            } else if response.hovered() {
                ui.painter()
                    .rect_filled(line, 1.0, theme::card_stroke(ui).color);
            }
            if response.clicked() && !selected {
                *value = *option;
                changed = true;
            }
        }
    });
    let bottom = ui.min_rect().bottom().max(top) + 7.0;
    let rect = ui.max_rect();
    ui.painter().hline(
        rect.x_range(),
        bottom,
        Stroke::new(1.0_f32, theme::card_stroke(ui).color),
    );
    ui.add_space(theme::LG);
    changed
}

/// One grouped control: `label [value] unit | action`. Returns true on action.
pub fn input_group(
    ui: &mut egui::Ui,
    label: &str,
    value: Option<(&mut String, &str)>,
    action: &str,
    help: &str,
) -> bool {
    let frame = egui::Frame::new()
        .fill(ui.visuals().widgets.inactive.weak_bg_fill)
        .stroke(ui.visuals().widgets.inactive.bg_stroke)
        .corner_radius(theme::RADIUS)
        .inner_margin(Margin::symmetric(10, 0));
    let inner = frame.show(ui, |ui| {
        ui.set_height(28.0);
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            ui.label(RichText::new(label).size(13.5));
            if let Some((text, unit)) = value {
                ui.add(
                    egui::TextEdit::singleline(text)
                        .desired_width(34.0)
                        .horizontal_align(egui::Align::Center)
                        .margin(Margin::symmetric(4, 2))
                        .background_color(theme::inset(ui)),
                );
                ui.label(RichText::new(unit).size(13.0).color(theme::muted(ui)));
            }
            ui.add(egui::Separator::default().vertical().spacing(6.0));
            let text = RichText::new(action)
                .size(13.5)
                .color(theme::accent())
                .strong();
            pointer(
                ui.add(egui::Label::new(text).sense(Sense::click()))
                    .on_hover_text(help),
            )
            .clicked()
        })
        .inner
    });
    inner.inner
}

/// Search box with a fixed width and hint.
pub fn search_field(
    ui: &mut egui::Ui,
    text: &mut String,
    hint: &str,
    width: f32,
) -> egui::Response {
    let hint = RichText::new(hint).color(theme::muted(ui));
    ui.add(
        egui::TextEdit::singleline(text)
            .hint_text(hint)
            .desired_width(width)
            .margin(Margin::symmetric(8, 5)),
    )
}
