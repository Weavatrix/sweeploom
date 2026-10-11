//! Layout helpers: inset panels, equal-height pairs and grouped inputs.

use eframe::egui::{self, Margin, RichText, Sense};

use super::pointer;

use crate::theme;

/// Inset sub-panel frame used inside cards.
pub fn panel_frame(ui: &egui::Ui) -> egui::Frame {
    let fill = if ui.visuals().dark_mode {
        egui::Color32::from_rgb(29, 33, 41)
    } else {
        egui::Color32::from_rgb(247, 248, 250)
    };
    egui::Frame::new()
        .fill(fill)
        .stroke(theme::card_stroke(ui))
        .corner_radius(8)
        .inner_margin(Margin::same(theme::MD as i8))
}

/// Title inside a panel or card.
pub fn panel_title(ui: &mut egui::Ui, title: &str) {
    ui.label(RichText::new(title).size(13.5).strong());
    ui.add_space(theme::SM);
}

/// Inset sub-panel inside a card: subtle fill, optional title, 12 px padding.
pub fn panel<R>(
    ui: &mut egui::Ui,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    panel_frame(ui).show(ui, |ui| {
        ui.set_width(ui.available_width());
        if !title.is_empty() {
            panel_title(ui, title);
        }
        add_contents(ui)
    })
}

type Side<'a> = Box<dyn FnOnce(&mut egui::Ui) + 'a>;

/// Two framed areas side by side with equal height (settles one frame late).
/// Stacked when the window is narrow.
pub fn pair(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash,
    frame: impl Fn(&egui::Ui) -> egui::Frame,
    left: impl FnOnce(&mut egui::Ui),
    right: impl FnOnce(&mut egui::Ui),
) {
    let sides: [Side<'_>; 2] = [Box::new(left), Box::new(right)];
    if ui.available_width() < 760.0 {
        for draw in sides {
            frame(ui).show(ui, |ui| {
                ui.set_width(ui.available_width());
                draw(ui);
            });
            ui.add_space(theme::MD);
        }
        return;
    }
    let id = ui.id().with(id);
    let min = ui.data(|data| data.get_temp::<f32>(id)).unwrap_or(0.0);
    let mut tallest = 0.0_f32;
    ui.columns(2, |columns| {
        for (column, draw) in columns.iter_mut().zip(sides) {
            frame(column).show(column, |ui| {
                ui.set_width(ui.available_width());
                let top = ui.cursor().top();
                ui.set_min_height(min);
                draw(ui);
                // The cursor tracks content only; min height does not move it.
                let natural = ui.cursor().top() - ui.spacing().item_spacing.y - top;
                tallest = tallest.max(natural);
            });
        }
    });
    if (tallest - min).abs() > 0.5 {
        ui.data_mut(|data| data.insert_temp(id, tallest));
        ui.ctx().request_repaint();
    }
    ui.add_space(theme::MD);
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
