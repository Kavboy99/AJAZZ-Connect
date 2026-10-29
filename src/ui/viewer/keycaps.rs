//! Replace baked key-top photography with independent, outline-rasterized legends.
use super::{
    glb::{Image, Scene},
    preview::binding,
};
use ab_glyph::{Font, FontArc, PxScale, ScaleFont, point};

const SIDE: usize = 2048;
const CELL: usize = 192;
const COLUMNS: usize = SIDE / CELL;

pub(super) fn prepare(scene: &mut Scene, visual: &str) {
    super::finish::housing(scene, visual);
    let Some(binding) = binding(Some(visual)) else {
        return;
    };
    let Some(font) = std::env::var_os("WINDIR")
        .and_then(|dir| std::fs::read(std::path::PathBuf::from(dir).join("Fonts/seguisb.ttf")).ok())
        .and_then(|bytes| FontArc::try_from_vec(bytes).ok())
    else {
        return;
    };
    let mut atlas = Image {
        width: SIDE as u32,
        height: SIDE as u32,
        rgba: vec![0; SIDE * SIDE * 4],
        legend: vec![0; SIDE * SIDE],
    };
    let texture = scene.images.len();
    let mut changed = false;
    for batch in &mut scene.batches {
        let Some(source) = batch.material.texture.and_then(|i| scene.images.get(i)) else {
            continue;
        };
        // A batch must contain only bound key surfaces; case textures remain untouched.
        if batch.vertices.is_empty()
            || batch.vertices.iter().any(|v| {
                scene.parts[v.part]
                    .as_deref()
                    .and_then(|n| binding.key_index(n))
                    .is_none()
            })
        {
            continue;
        }
        let parts: std::collections::BTreeSet<_> = batch.vertices.iter().map(|v| v.part).collect();
        for part in parts {
            let Some(index) = scene.parts[part]
                .as_deref()
                .and_then(|n| binding.key_index(n))
            else {
                continue;
            };
            let Some(label) = binding.key_label(index) else {
                continue;
            };
            let origin = [(index % COLUMNS) * CELL, (index / COLUMNS) * CELL];
            if origin[1] + CELL > SIDE {
                continue;
            }
            let mut lo = [f32::INFINITY; 2];
            let mut hi = [f32::NEG_INFINITY; 2];
            let mut uv_lo = [f32::INFINITY; 2];
            let mut uv_hi = [f32::NEG_INFINITY; 2];
            let mut samples = [Vec::new(), Vec::new(), Vec::new()];
            for v in batch.vertices.iter().filter(|v| v.part == part) {
                for (i, axis) in [0, 2].into_iter().enumerate() {
                    lo[i] = lo[i].min(v.position[axis]);
                    hi[i] = hi[i].max(v.position[axis]);
                    uv_lo[i] = uv_lo[i].min(v.uv[i]);
                    uv_hi[i] = uv_hi[i].max(v.uv[i]);
                }
            }
            for row in 0..7 {
                for column in 0..7 {
                    let uv: [f32; 2] = std::array::from_fn(|i| {
                        uv_lo[i] + (uv_hi[i] - uv_lo[i]) * (0.25 + [column, row][i] as f32 / 12.0)
                    });
                    let x = (uv[0] * source.width as f32).clamp(0.0, source.width as f32 - 1.0)
                        as usize;
                    let y = (uv[1] * source.height as f32).clamp(0.0, source.height as f32 - 1.0)
                        as usize;
                    let at = (y * source.width as usize + x) * 4;
                    for (c, values) in samples.iter_mut().enumerate() {
                        values.push(source.rgba[at + c]);
                    }
                }
            }
            let sampled = samples.map(|mut values| {
                values.sort_unstable();
                values[values.len() / 2]
            });
            let color = super::finish::plastic(visual, label, sampled);
            for y in origin[1]..origin[1] + CELL {
                for x in origin[0]..origin[0] + CELL {
                    let at = (y * SIDE + x) * 4;
                    atlas.rgba[at..at + 4].copy_from_slice(&[color[0], color[1], color[2], 255]);
                }
            }
            let aspect = ((hi[0] - lo[0]) / (hi[1] - lo[1]).max(0.001)).max(1.0);
            if visual == "ak820-white" {
                super::finish::contours(&mut atlas, origin, CELL, SIDE, index);
            }
            lettering(&mut atlas, &font, label, origin, aspect, color);
            for v in batch.vertices.iter_mut().filter(|v| v.part == part) {
                v.uv = std::array::from_fn(|i| {
                    let coordinate = (v.position[[0, 2][i]] - lo[i]) / (hi[i] - lo[i]).max(0.001);
                    (origin[i] as f32 + 4.0 + coordinate * (CELL - 8) as f32) / SIDE as f32
                });
            }
        }
        batch.material.texture = Some(texture);
        batch.material.color = [1.0; 4];
        batch.material.unlit = false;
        batch.material.roughness = 0.48;
        changed = true;
    }
    if changed {
        scene.images.push(atlas);
    }
}

fn lettering(
    atlas: &mut Image,
    font: &FontArc,
    label: &str,
    origin: [usize; 2],
    aspect: f32,
    color: [u8; 3],
) {
    let label = match label {
        "Escape" => "Esc",
        "Backtick" => "~",
        "Minus" => "−",
        "Equals" => "+",
        "OpenBracket" => "[",
        "CloseBracket" => "]",
        "Backslash" => "\\",
        "Semicolon" => ";",
        "Quote" => "'",
        "Comma" => ",",
        "Period" => ".",
        "Slash" => "/",
        "ArrowUp" => "↑",
        "ArrowDown" => "↓",
        "ArrowLeft" => "←",
        "ArrowRight" => "→",
        "PageUp" => "PgUp",
        "PageDown" => "PgDn",
        "CapsLock" => "Caps",
        "Super" => "Win",
        "Backspace" => "Backspace",
        "Space" => "",
        other => other,
    };
    let scale = PxScale::from(64.0);
    let scaled = font.as_scaled(scale);
    let width: f32 = label
        .chars()
        .map(|c| scaled.h_advance(scaled.glyph_id(c)))
        .sum();
    let squeeze = (150.0 * aspect / width.max(1.0)).min(1.0) / aspect;
    // Rasterize at the final horizontal scale so narrow labels never overwrite pixels.
    let scale = PxScale {
        x: scale.x * squeeze,
        y: scale.y,
    };
    let scaled = font.as_scaled(scale);
    let width = width * squeeze;
    let mut cursor = 0.0;
    let ink: [u8; 3] = if color == [246, 245, 242] {
        [139, 113, 172]
    } else if color == [174, 149, 202] || color == [184, 215, 234] {
        [250, 249, 247]
    } else if u16::from(color[0]) + u16::from(color[1]) + u16::from(color[2]) > 500 {
        [49, 51, 45]
    } else {
        [232, 232, 214]
    };
    for c in label.chars() {
        let glyph = scaled
            .glyph_id(c)
            .with_scale_and_position(scale, point(cursor, 0.0));
        cursor += scaled.h_advance(glyph.id);
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|x, y, coverage| {
                let x =
                    origin[0] as f32 + CELL as f32 * 0.5 + (bounds.min.x + x as f32 - width * 0.5);
                let y = origin[1] as f32 + CELL as f32 * 0.5 + bounds.min.y + y as f32 + 22.0;
                let x = x.round() as usize;
                let y = y.round() as usize;
                if x >= SIDE || y >= SIDE {
                    return;
                }
                let at = y * SIDE + x;
                atlas.legend[at] = atlas.legend[at].max((coverage * 255.0) as u8);
                for (c, channel) in ink.iter().enumerate() {
                    let base = atlas.rgba[at * 4 + c] as f32;
                    atlas.rgba[at * 4 + c] =
                        (base * (1.0 - coverage) + f32::from(*channel) * coverage) as u8;
                }
            });
        }
    }
}
