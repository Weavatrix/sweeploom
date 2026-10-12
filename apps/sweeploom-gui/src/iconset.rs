//! `sweeploom-gui --write-iconset DIR`: the app icon as the PNG set `iconutil`
//! turns into `AppIcon.icns` (used by `scripts/macos-mas.py`).
//!
//! The loom mark sits on a dark rounded plate on the macOS icon grid
//! (824 of 1024 px). Everything renders once at 2048 px and is downsampled,
//! which anti-aliases the mark's hard edges.

use std::path::Path;

use image::{Rgba, RgbaImage, imageops};

const MASTER: u32 = 2048;
/// `iconutil` names and pixel sizes.
const SIZES: &[(u32, &str)] = &[
    (16, "icon_16x16"),
    (32, "icon_16x16@2x"),
    (32, "icon_32x32"),
    (64, "icon_32x32@2x"),
    (128, "icon_128x128"),
    (256, "icon_128x128@2x"),
    (256, "icon_256x256"),
    (512, "icon_256x256@2x"),
    (512, "icon_512x512"),
    (1024, "icon_512x512@2x"),
];

/// Write every icon size into `dir` (created when missing).
pub fn write(dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let master = master()?;
    for (size, name) in SIZES {
        imageops::resize(&master, *size, *size, imageops::FilterType::Triangle)
            .save(dir.join(format!("{name}.png")))
            .map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}

fn master() -> Result<RgbaImage, String> {
    let n = MASTER as f32;
    let margin = n * 100.0 / 1024.0;
    let side = n - 2.0 * margin;
    let mut image = RgbaImage::new(MASTER, MASTER);
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let coverage = plate_coverage(x as f32 + 0.5, y as f32 + 0.5, n / 2.0, side);
        if coverage > 0.0 {
            // Header chrome at the bottom, a little lighter at the top.
            let t = ((y as f32 - margin) / side).clamp(0.0, 1.0);
            let shade = |top: f32, bottom: f32| (top + (bottom - top) * t).round() as u8;
            *pixel = Rgba([
                shade(40.0, 18.0),
                shade(44.0, 20.0),
                shade(52.0, 24.0),
                (coverage * 255.0).round() as u8,
            ]);
        }
    }
    let mark_side = (side * 0.74).round() as u32;
    let (rgba, width, height) = crate::mark::rgba(mark_side);
    let mark = RgbaImage::from_raw(width, height, rgba).ok_or("mark buffer size mismatch")?;
    let offset = i64::from((MASTER - width) / 2);
    imageops::overlay(&mut image, &mark, offset, offset);
    Ok(image)
}

/// Coverage of a rounded square centred at `center` with edge `side`.
fn plate_coverage(x: f32, y: f32, center: f32, side: f32) -> f32 {
    let radius = side * 0.2237;
    let inner = side / 2.0 - radius;
    let dx = ((x - center).abs() - inner).max(0.0);
    let dy = ((y - center).abs() - inner).max(0.0);
    let distance = (dx * dx + dy * dy).sqrt() - radius;
    (0.5 - distance).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plate_is_opaque_inside_and_clear_outside() {
        let side = 824.0;
        assert!((plate_coverage(512.0, 512.0, 512.0, side) - 1.0).abs() < f32::EPSILON);
        assert!(plate_coverage(2.0, 2.0, 512.0, side) < f32::EPSILON);
        // Rounded corner: the square's corner point is outside the plate.
        assert!(plate_coverage(101.0, 101.0, 512.0, side) < f32::EPSILON);
    }

    #[test]
    fn iconset_has_every_size_iconutil_expects() {
        let dir = std::env::temp_dir().join(format!("sweeploom-iconset-{}", std::process::id()));
        write(&dir).unwrap();
        for (size, name) in SIZES {
            let image = image::open(dir.join(format!("{name}.png"))).unwrap();
            assert_eq!((image.width(), image.height()), (*size, *size));
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
}
