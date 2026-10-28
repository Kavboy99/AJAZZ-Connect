//! Optical diffusion from visible LED emission only, never from white plastic.
use super::{ColorImage, SRGB_STEPS, tables};
use eframe::egui::Color32;
pub(super) struct Band {
    pub pixels: Vec<Color32>,
    pub glow: Vec<[f32; 3]>,
}

pub(super) fn finish(side: usize, band: Band) -> ColorImage {
    if band.glow.is_empty() {
        return ColorImage::new([side, side], band.pixels);
    }
    const DOWN: usize = 4;
    let small = side.div_ceil(DOWN);
    let mut light = vec![[0.0; 3]; small * small];
    for y in 0..side {
        for x in 0..side {
            for (out, source) in light[(y / DOWN) * small + x / DOWN]
                .iter_mut()
                .zip(band.glow[y * side + x])
            {
                *out += source / (DOWN * DOWN) as f32;
            }
        }
    }
    let kernel = [1.0, 4.0, 7.0, 10.0, 13.0, 10.0, 7.0, 4.0, 1.0];
    let blur = |source: &[[f32; 3]], horizontal: bool| {
        let mut output = vec![[0.0; 3]; small * small];
        for y in 0..small {
            for x in 0..small {
                for (i, weight) in kernel.iter().enumerate() {
                    let offset = i as isize - 4;
                    let sx = if horizontal {
                        (x as isize + offset).clamp(0, small as isize - 1) as usize
                    } else {
                        x
                    };
                    let sy = if horizontal {
                        y
                    } else {
                        (y as isize + offset).clamp(0, small as isize - 1) as usize
                    };
                    for c in 0..3 {
                        output[y * small + x][c] += source[sy * small + sx][c] * weight / 57.0;
                    }
                }
            }
        }
        output
    };
    let light = blur(&blur(&light, true), false);
    let lookup = tables();
    let mut pixels = band.pixels;
    for y in 0..side {
        for x in 0..side {
            let px = x as f32 / DOWN as f32;
            let py = y as f32 / DOWN as f32;
            let (x0, y0) = (px as usize, py as usize);
            let (x1, y1) = ((x0 + 1).min(small - 1), (y0 + 1).min(small - 1));
            let p = pixels[y * side + x];
            let base = [p.r(), p.g(), p.b()];
            let rgb: [u8; 3] = std::array::from_fn(|c| {
                let a = light[y0 * small + x0][c] * (1.0 - px.fract())
                    + light[y0 * small + x1][c] * px.fract();
                let b = light[y1 * small + x0][c] * (1.0 - px.fract())
                    + light[y1 * small + x1][c] * px.fract();
                let value = lookup.linear[base[c] as usize]
                    + (a * (1.0 - py.fract()) + b * py.fract()) * 0.12;
                lookup.srgb[(value.clamp(0.0, 1.0) * (SRGB_STEPS - 1) as f32) as usize]
            });
            pixels[y * side + x] = Color32::from_rgba_premultiplied(rgb[0], rgb[1], rgb[2], p.a());
        }
    }
    ColorImage::new([side, side], pixels)
}
