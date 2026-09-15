//! SweepLoom mark as RGBA. Shared by the window and the tray.

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};

/// Gold loom on a transparent field. Not a filled square.
#[must_use]
pub fn rgba(size: u32) -> (Vec<u8>, u32, u32) {
    let n = size.max(16);
    let mut pixels = vec![0_u8; (n * n * 4) as usize];
    let gold = [196_u8, 140, 64, 255];
    let ink = [92_u8, 62, 24, 255];
    for y in 0..n {
        for x in 0..n {
            let u = (x as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let v = (y as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let r = (u * u + v * v).sqrt();
            let ring = (r - 0.68).abs() < 0.10 && r < 0.92;
            let warp = (u - v * 0.15).abs() < 0.09 && r < 0.62;
            let weft = (v + u * 0.12).abs() < 0.09 && r < 0.62;
            let hub = r < 0.12;
            if ring || warp || weft || hub {
                let i = ((y * n + x) * 4) as usize;
                let color = if hub { ink } else { gold };
                pixels[i..i + 4].copy_from_slice(&color);
            }
        }
    }
    (pixels, n, n)
}

/// Header / chrome mark.
pub fn show(ui: &mut egui::Ui, size: f32) {
    let texture = texture(ui.ctx());
    let side = size.max(16.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::hover());
    egui::Image::new((texture.id(), egui::vec2(side, side)))
        .fit_to_exact_size(egui::vec2(side, side))
        .paint_at(ui, rect);
}

fn texture(ctx: &egui::Context) -> TextureHandle {
    let key = egui::Id::new("sweeploom-mark");
    if let Some(handle) = ctx.data(|data| data.get_temp::<TextureHandle>(key)) {
        return handle;
    }
    let (rgba, width, height) = rgba(64);
    let image = ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &rgba);
    let handle = ctx.load_texture("sweeploom-mark", image, TextureOptions::LINEAR);
    ctx.data_mut(|data| data.insert_temp(key, handle.clone()));
    handle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_stay_transparent() {
        let (pixels, n, _) = rgba(32);
        let last = ((n * n - 1) * 4 + 3) as usize;
        assert_eq!(pixels[3], 0);
        assert_eq!(pixels[last], 0);
    }

    #[test]
    fn mark_uses_gold_not_a_solid_fill() {
        let (pixels, n, _) = rgba(32);
        let gold = pixels.chunks(4).filter(|px| px[3] > 0).count();
        let total = (n * n) as usize;
        assert!(gold > 40);
        assert!(gold < total / 2);
    }
}
