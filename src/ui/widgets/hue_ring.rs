//! Analytic, DPI-aware coverage. The spectrum is immutable while a marker is dragged.
use eframe::egui::{Color32, ColorImage, Id, Rect, TextureHandle, TextureOptions, Ui, pos2};
use std::f32::consts::TAU;

#[derive(Clone)]
struct Cached {
    key: [u32; 3],
    texture: TextureHandle,
}

pub(super) fn show(
    ui: &Ui,
    id: Id,
    rect: Rect,
    count: usize,
    outer: f32,
    thickness: f32,
    stride: f32,
) {
    let density = ui.ctx().pixels_per_point().max(2.0);
    let side = (rect.width() * density).ceil() as usize;
    let key = [side as u32, count as u32, (density * 100.0).round() as u32];
    let previous = ui.ctx().data(|d| d.get_temp::<Cached>(id));
    let texture = if let Some(cached) = previous.filter(|c| c.key == key) {
        cached.texture
    } else {
        let texture = ui.ctx().load_texture(
            "antialiased hue rings",
            raster(side, density, count, outer, thickness, stride),
            TextureOptions::LINEAR,
        );
        ui.ctx().data_mut(|d| {
            d.insert_temp(
                id,
                Cached {
                    key,
                    texture: texture.clone(),
                },
            )
        });
        texture
    };
    ui.painter().image(
        texture.id(),
        rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::WHITE,
    );
}

fn raster(
    side: usize,
    density: f32,
    count: usize,
    outer: f32,
    thickness: f32,
    stride: f32,
) -> ColorImage {
    let mut pixels = vec![Color32::TRANSPARENT; side * side];
    for y in 0..side {
        for x in 0..side {
            let dx = (x as f32 + 0.5 - side as f32 * 0.5) / density;
            let dy = (y as f32 + 0.5 - side as f32 * 0.5) / density;
            let radius = dx.hypot(dy);
            let coverage = (0..count)
                .map(|ring| {
                    let r = outer - ring as f32 * stride;
                    ((r - radius).min(radius - (r - thickness)) * density + 0.5).clamp(0.0, 1.0)
                })
                .fold(0.0_f32, f32::max);
            if coverage > 0.0 {
                let hue = (dy.atan2(dx) / TAU).rem_euclid(1.0);
                let c: Color32 = eframe::egui::ecolor::Hsva::new(hue, 1.0, 1.0, 1.0).into();
                pixels[y * side + x] = Color32::from_rgba_unmultiplied(
                    c.r(),
                    c.g(),
                    c.b(),
                    (coverage * 255.0).round() as u8,
                );
            }
        }
    }
    ColorImage::new([side, side], pixels)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ring_has_subpixel_edges_and_a_transparent_center() {
        let image = raster(560, 2.0, 3, 132.0, 20.0, 28.0);
        assert_eq!(image.pixels[280 * 560 + 280], Color32::TRANSPARENT);
        assert!(image.pixels.iter().any(|c| c.a() > 0 && c.a() < 255));
        assert!(image.pixels.iter().any(|c| c.a() == 255));
    }
}
