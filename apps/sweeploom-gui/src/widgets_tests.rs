use super::*;
use egui_extras::Column;

#[test]
fn table_keeps_size_visible_after_window_shrinks() {
    let ctx = egui::Context::default();
    for width in [1800.0, 900.0, 1300.0] {
        let mut right = 0.0;
        let mut limit = 0.0;
        for _ in 0..2 {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 400.0),
                )),
                ..Default::default()
            };
            let _ = ctx.run(input, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    limit = ui.clip_rect().right();
                    table(ui, "resize-test")
                        .resizable(true)
                        .column(Column::remainder().at_least(160.0).clip(true))
                        .column(Column::exact(72.0))
                        .column(Column::exact(168.0))
                        .body(|mut body| {
                            body.row(TABLE_ROW, |mut row| {
                                row.col(|ui| {
                                    ui.add(egui::Label::new("long-name".repeat(200)).truncate());
                                });
                                row.col(|ui| {
                                    ui.label("12 MB");
                                });
                                right = row
                                    .col(|ui| {
                                        ui.label("Inspect only");
                                    })
                                    .1
                                    .rect
                                    .right();
                            });
                        });
                });
            });
        }
        assert!(
            right <= limit + 1.0,
            "width={width}, right={right}, limit={limit}"
        );
    }
}

#[test]
fn checkbox_toggles_and_disabled_stays_put() {
    let ctx = egui::Context::default();
    let mut on = false;
    let mut off = false;
    let mut rects = (egui::Rect::NOTHING, egui::Rect::NOTHING);
    let click = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            },
        ]
    };
    for step in 0..3 {
        let events = match step {
            1 => click(rects.0.center()),
            2 => click(rects.1.center()),
            _ => Vec::new(),
        };
        let input = egui::RawInput {
            events,
            ..Default::default()
        };
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rects.0 = check(ui, &mut on).rect;
                rects.1 = check_enabled(ui, &mut off, false).rect;
            });
        });
    }
    assert!(on);
    assert!(!off);
}
