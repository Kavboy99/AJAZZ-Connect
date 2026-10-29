//! Plastic colors and flowing printed artwork, independent of light emission.
use super::glb::Image;

pub(super) fn plastic(visual: &str, label: &str, original: [u8; 3]) -> [u8; 3] {
    match visual {
        "ak820-white" => [244, 244, 241],
        "ak820-purple" => match label {
            "Escape" | "Enter" | "Space" => [184, 215, 234],
            "F5" | "F6" | "F7" | "F8" | "Delete" | "Backspace" | "Tab" | "CapsLock" | "Shift"
            | "Ctrl" | "Super" | "Alt" | "Fn" | "ArrowUp" | "ArrowDown" | "ArrowLeft"
            | "ArrowRight" | "Home" | "PageUp" | "PageDown" | "End" => [174, 149, 202],
            _ => [246, 245, 242],
        },
        _ => original,
    }
}

pub(super) fn contours(
    image: &mut Image,
    origin: [usize; 2],
    cell: usize,
    side: usize,
    key: usize,
) {
    for y in 0..cell {
        for x in 0..cell {
            let u = x as f32 + (key % 14) as f32 * 137.0;
            let v = y as f32 + (key / 14) as f32 * 151.0;
            let field = u * 0.045 + (v * 0.040).sin() * 2.8 + ((u + v) * 0.013).sin() * 1.2;
            let coverage = (1.0 - (field.sin().abs() * 16.0).min(1.0)) * 0.95;
            let at = ((origin[1] + y) * side + origin[0] + x) * 4;
            for c in 0..3 {
                image.rgba[at + c] =
                    (f32::from(image.rgba[at + c]) * (1.0 - coverage) + 70.0 * coverage) as u8;
            }
        }
    }
}

pub(super) fn housing(scene: &mut super::glb::Scene, visual: &str) {
    if !matches!(visual, "ak820-purple" | "ak820-white") {
        return;
    }
    for batch in &mut scene.batches {
        if !batch.vertices.is_empty()
            && batch.vertices.iter().all(|v| {
                matches!(
                    scene.parts[v.part].as_deref(),
                    Some("Lower housing" | "Upper rim" | "Cream center inset")
                )
            })
        {
            batch.material.texture = None;
            batch.material.color = [0.88, 0.88, 0.86, 1.0];
            batch.material.unlit = false;
        }
    }
}
