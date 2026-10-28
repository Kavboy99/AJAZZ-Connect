//! Prepare light transport and legend apertures once, in model space.
use super::glb::Scene;
use super::math::{self, Vec3};
use super::preview::binding;

#[derive(Clone, Copy)]
struct Bounds {
    lo: Vec3,
    hi: Vec3,
    uv_lo: [f32; 2],
    uv_hi: [f32; 2],
}
impl Default for Bounds {
    fn default() -> Self {
        Self {
            lo: [f32::INFINITY; 3],
            hi: [f32::NEG_INFINITY; 3],
            uv_lo: [f32::INFINITY; 2],
            uv_hi: [f32::NEG_INFINITY; 2],
        }
    }
}
impl Bounds {
    fn center(self) -> Vec3 {
        std::array::from_fn(|i| (self.lo[i] + self.hi[i]) * 0.5)
    }
}

pub(super) fn prepare(scene: &mut Scene, visual: &str) {
    let Some(binding) = binding(Some(visual)) else {
        return;
    };
    let mut bounds = vec![Bounds::default(); scene.parts.len()];
    for v in scene.batches.iter().flat_map(|b| &b.vertices) {
        let b = &mut bounds[v.part];
        for i in 0..3 {
            b.lo[i] = b.lo[i].min(v.position[i]);
            b.hi[i] = b.hi[i].max(v.position[i]);
        }
        for i in 0..2 {
            b.uv_lo[i] = b.uv_lo[i].min(v.uv[i]);
            b.uv_hi[i] = b.uv_hi[i].max(v.uv[i]);
        }
    }
    let keys: Vec<_> = scene
        .parts
        .iter()
        .enumerate()
        .filter_map(|(part, name)| Some((part, binding.key_index(name.as_deref()?)?)))
        .collect();
    let sockets: Vec<_> = scene
        .parts
        .iter()
        .enumerate()
        .filter_map(|(part, name)| {
            let index = binding.socket_index(name.as_deref()?)?;
            let key = keys.iter().find(|(_, key)| *key == index)?.0;
            let mut center = bounds[key].center();
            // LED emitters sit below the cap, so the top face receives no diffuse wash.
            center[1] = bounds[key].lo[1] - 0.04;
            Some((part, center))
        })
        .collect();
    for batch in &mut scene.batches {
        for v in &mut batch.vertices {
            v.legend = if scene.parts[v.part]
                .as_deref()
                .is_some_and(|name| binding.key_index(name).is_some())
            {
                ((v.normal[1] - 0.65) / 0.35).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let mut nearest = [(0, 0.0); 4];
            for &(part, position) in &sockets {
                let delta = math::sub(position, v.position);
                let distance2 = math::dot(delta, delta).max(0.02);
                if distance2 > 4.0 {
                    continue;
                }
                let cosine = math::dot(v.normal, math::normalize(delta)).max(0.0);
                let weight = cosine * 0.28 / (0.12 + distance2 * 3.0);
                if weight > nearest[3].1 {
                    nearest[3] = (part, weight);
                    nearest.sort_by(|a, b| b.1.total_cmp(&a.1));
                }
            }
            v.bounce = nearest;
        }
        let Some(image_index) = batch.material.texture else {
            continue;
        };
        let image = &mut scene.images[image_index];
        if image.legend.is_empty() {
            image
                .legend
                .resize(image.width as usize * image.height as usize, 0);
        }
        for &(part, _) in &keys {
            if !batch.vertices.iter().any(|v| v.part == part) {
                continue;
            }
            let b = bounds[part];
            let w = image.width as usize;
            let h = image.height as usize;
            let x0 = (b.uv_lo[0] * w as f32).max(0.0) as usize;
            let x1 = (b.uv_hi[0] * w as f32).min(w as f32) as usize;
            let y0 = (b.uv_lo[1] * h as f32).max(0.0) as usize;
            let y1 = (b.uv_hi[1] * h as f32).min(h as f32) as usize;
            if x1 <= x0 || y1 <= y0 {
                continue;
            }
            let inset_x = ((x1 - x0) as f32 * 0.12) as usize;
            let inset_y = ((y1 - y0) as f32 * 0.18) as usize;
            let mut histogram = [0usize; 256];
            let mut red = 0u64;
            let mut green = 0u64;
            let mut samples = 0usize;
            for y in y0 + inset_y..y1 - inset_y {
                for x in x0 + inset_x..x1 - inset_x {
                    let p = &image.rgba[(y * w + x) * 4..][..4];
                    histogram[luma(p) as usize] += 1;
                    red += u64::from(p[0]);
                    green += u64::from(p[1]);
                    samples += 1;
                }
            }
            let mut total = 0;
            let median = histogram
                .iter()
                .position(|count| {
                    total += count;
                    total >= samples / 2
                })
                .unwrap_or(128) as f32;
            let light_legend = median < 200.0 || red > green * 6 / 5;
            for y in y0 + inset_y..y1 - inset_y {
                for x in x0 + inset_x..x1 - inset_x {
                    let at = y * w + x;
                    let value = f32::from(luma(&image.rgba[at * 4..][..4]));
                    let contrast = if light_legend {
                        value - median
                    } else {
                        median - value
                    };
                    image.legend[at] = (((contrast - 25.0) / 45.0).clamp(0.0, 1.0) * 255.0) as u8;
                }
            }
        }
    }
}

fn luma(p: &[u8]) -> u8 {
    ((u16::from(p[0]) * 54 + u16::from(p[1]) * 183 + u16::from(p[2]) * 19) / 256) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_legends_and_local_light_transport_are_retained() {
        let bytes = super::super::catalog::model_bytes("ak820-max").unwrap();
        let mut scene = super::super::glb::parse(bytes).unwrap();
        assert_eq!(scene.images[0].width, 1536);
        prepare(&mut scene, "ak820-max");
        let image = &scene.images[0];
        let apertures = image.legend.iter().filter(|v| **v > 0).count();
        assert!(
            apertures > 100 && apertures < image.legend.len() / 10,
            "legend mask must cover glyphs only: {apertures}"
        );
        assert!(
            scene
                .batches
                .iter()
                .flat_map(|b| &b.vertices)
                .any(|v| v.bounce.iter().any(|(_, weight)| *weight > 0.0))
        );
        assert!(
            scene
                .batches
                .iter()
                .flat_map(|b| &b.vertices)
                .any(|v| v.normal[1] > 0.99 && v.bounce.iter().all(|(_, weight)| *weight == 0.0))
        );
    }
}
