//! A small depth-buffered triangle rasterizer. It draws a device model into an image that the
//! interface shows as an ordinary texture, so the 3D view needs no GPU code (and no `unsafe`),
//! and `--review` screenshots show exactly what the window shows.

use super::glb::{Image, Scene};
use super::light::{Lit, Rig, to_linear, to_srgb};
use super::math::{self, Vec3};
use eframe::egui::ColorImage;
use std::sync::OnceLock;
mod bloom;
mod camera;
mod target;
use camera::Camera;
use target::Target;

const SRGB_STEPS: usize = 4096;

/// How the model is held. Angles in radians; `lift` and `glow` in 0..1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub zoom: f32,
    pub focus: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub lift: f32,
    pub glow: f32,
    /// Linear RGB of the rim light at full glow.
    pub glow_color: Vec3,
}

struct Tables {
    /// sRGB byte to linear.
    linear: [f32; 256],
    /// Linear 0..1 in `SRGB_STEPS` steps to an sRGB byte.
    srgb: Vec<u8>,
}

/// Built once: a lookup instead of a power function per texel and per pixel.
fn tables() -> &'static Tables {
    static TABLES: OnceLock<Tables> = OnceLock::new();
    TABLES.get_or_init(|| Tables {
        linear: std::array::from_fn(|i| to_linear(i as f32 / 255.0)),
        srgb: (0..SRGB_STEPS)
            .map(|i| (to_srgb(i as f32 / (SRGB_STEPS - 1) as f32) * 255.0).round() as u8)
            .collect(),
    })
}

struct Projected {
    x: f32,
    y: f32,
    z: f32,
    inv_w: f32,
    uv: [f32; 2],
    lit: Lit,
}

fn sample(image: &Image, linear: &[f32; 256], uv: [f32; 2]) -> ([f32; 4], f32) {
    let (w, h) = (image.width as usize, image.height as usize);
    let x = (uv[0].clamp(0.0, 1.0) * w as f32 - 0.5).clamp(0.0, (w - 1) as f32);
    let y = (uv[1].clamp(0.0, 1.0) * h as f32 - 0.5).clamp(0.0, (h - 1) as f32);
    let (x0, y0) = (x as usize, y as usize);
    let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let corners = [
        (y0 * w + x0) * 4,
        (y0 * w + x1) * 4,
        (y1 * w + x0) * 4,
        (y1 * w + x1) * 4,
    ];
    let rgba = std::array::from_fn(|c| {
        let texel = |at: usize| {
            let byte = image.rgba[at + c];
            if c == 3 {
                f32::from(byte) / 255.0
            } else {
                linear[usize::from(byte)]
            }
        };
        let top = texel(corners[0]) + (texel(corners[1]) - texel(corners[0])) * fx;
        let bottom = texel(corners[2]) + (texel(corners[3]) - texel(corners[2])) * fx;
        top + (bottom - top) * fy
    });
    let mask = |at: usize| image.legend.get(at / 4).copied().unwrap_or(0) as f32 / 255.0;
    let top = mask(corners[0]) + (mask(corners[1]) - mask(corners[0])) * fx;
    let bottom = mask(corners[2]) + (mask(corners[3]) - mask(corners[2])) * fx;
    (rgba, top + (bottom - top) * fy)
}

struct Fill<'a> {
    texture: Option<&'a Image>,
    alpha: f32,
    blend: bool,
}

/// `w = a·x + b·y + c`: one barycentric weight as a plane over the screen.
#[derive(Clone, Copy)]
struct Edge {
    a: f32,
    b: f32,
    c: f32,
}

impl Edge {
    fn new(p: &Projected, q: &Projected, area: f32) -> Self {
        Self {
            a: -(q.y - p.y) / area,
            b: (q.x - p.x) / area,
            c: ((q.y - p.y) * p.x - (q.x - p.x) * p.y) / area,
        }
    }

    fn at(self, x: f32, y: f32) -> f32 {
        self.a * x + self.b * y + self.c
    }
}

/// The model in a square of `side` pixels on a transparent background.
pub fn render(
    scene: &Scene,
    pose: &Pose,
    side: usize,
    effects: &[super::preview::PartEffect],
) -> ColorImage {
    let workers = std::thread::available_parallelism()
        .map_or(1, usize::from)
        .min(8);
    render_with_workers(
        scene,
        pose,
        side,
        effects,
        if side >= 400 { workers } else { 1 },
    )
}

struct Draw<'a> {
    projected: Vec<Projected>,
    indices: &'a [u32],
    fill: Fill<'a>,
}

fn render_with_workers(
    scene: &Scene,
    pose: &Pose,
    side: usize,
    effects: &[super::preview::PartEffect],
    workers: usize,
) -> ColorImage {
    let n = 2;
    let samples = side * n;
    let camera = Camera::new(scene, pose, samples);
    let rig = Rig::new(pose.glow_color.map(|c| c * pose.glow));
    let mut draws = Vec::with_capacity(scene.batches.len());
    for batch in &scene.batches {
        let m = &batch.material;
        let surface = rig.surface(m);
        let projected: Vec<Projected> = batch
            .vertices
            .iter()
            .map(|v| {
                let effect = effects.get(v.part).copied().unwrap_or_default();
                let mut position = v.position;
                position[1] -= effect.press * scene.radius * 0.008;
                let [x, y, z, inv_w] = camera.project(position);
                let normal = math::transform_direction(&camera.model, v.normal);
                let mut lit = rig.shade(normal, &surface);
                for c in 0..3 {
                    lit.emission[c] = effect.emission[c] * v.legend;
                    if effect.socket {
                        lit.add[c] += effect.emission[c];
                        lit.glow[c] = effect.emission[c];
                    }
                    for (part, weight) in v.bounce {
                        if let Some(source) = effects.get(part) {
                            lit.add[c] += source.emission[c] * weight * m.color[c];
                        }
                    }
                }
                Projected {
                    x,
                    y,
                    z,
                    inv_w,
                    uv: v.uv,
                    lit,
                }
            })
            .collect();
        let fill = Fill {
            texture: m.texture.and_then(|t| scene.images.get(t)),
            alpha: m.color[3],
            blend: m.blend,
        };
        draws.push(Draw {
            projected,
            indices: &batch.indices,
            fill,
        });
    }
    let band = |start: usize, rows: usize| {
        let emission = effects.iter().any(|e| e.emission.iter().any(|v| *v > 0.0));
        let mut target = Target::new(samples, start * n, rows * n, emission);
        for draw in &draws {
            for t in draw.indices.as_chunks::<3>().0 {
                let corner = |i: u32| &draw.projected[i as usize];
                target.triangle([corner(t[0]), corner(t[1]), corner(t[2])], &draw.fill);
            }
        }
        target.resolve(side, n)
    };
    let result = if workers <= 1 {
        Some(band(0, side))
    } else {
        // Bands share immutable geometry and retain triangle order. Threads exist only for this frame.
        std::thread::scope(|scope| {
            let rows = side.div_ceil(workers);
            let jobs: Vec<_> = (0..side)
                .step_by(rows)
                .map(|start| {
                    let band = &band;
                    scope.spawn(move || band(start, rows.min(side - start)))
                })
                .collect();
            let mut result = bloom::Band {
                pixels: Vec::with_capacity(side * side),
                glow: Vec::new(),
            };
            for job in jobs {
                match job.join() {
                    Ok(part) => {
                        result.pixels.extend(part.pixels);
                        result.glow.extend(part.glow);
                    }
                    Err(_) => return None,
                }
            }
            Some(result)
        })
    };
    match result {
        Some(band) => bloom::finish(side, band),
        None => render_with_workers(scene, pose, side, effects, 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bands_preserve_serial_pixels() {
        let bytes = super::super::catalog::model_bytes("ak820-max").unwrap();
        let scene = super::super::glb::parse(bytes).unwrap();
        let pose = Pose {
            zoom: 1.0,
            focus: [0.0; 3],
            yaw: -0.35,
            pitch: 0.9,
            lift: 0.0,
            glow: 0.0,
            glow_color: [0.0; 3],
        };
        assert_eq!(
            render_with_workers(&scene, &pose, 96, &[], 1).pixels,
            render_with_workers(&scene, &pose, 96, &[], 4).pixels
        );
    }
    #[test]
    fn emissive_legends_and_bloom_have_no_band_seams() {
        let bytes = super::super::catalog::model_bytes("ak820-max").unwrap();
        let mut scene = super::super::glb::parse(bytes).unwrap();
        super::super::led::prepare(&mut scene, "ak820-max");
        let preview = super::super::preview::Preview {
            native: None,
            brightness: 1.0,
            effect: Some(super::super::preview::Effect::Solid),
            colors: vec![eframe::egui::Color32::from_rgb(0, 190, 255)],
            phase: 0.0,
            keys: Default::default(),
            modifiers: Default::default(),
        };
        let effects = super::super::preview::binding(Some("ak820-max"))
            .unwrap()
            .effects(&scene, &preview);
        let pose = Pose {
            zoom: 1.0,
            focus: [0.0; 3],
            yaw: 0.0,
            pitch: 1.25,
            lift: 0.0,
            glow: 0.0,
            glow_color: [0.0; 3],
        };
        let lit = render_with_workers(&scene, &pose, 128, &effects, 1);
        assert_eq!(
            lit.pixels,
            render_with_workers(&scene, &pose, 128, &effects, 4).pixels
        );
        assert_ne!(
            lit.pixels,
            render_with_workers(&scene, &pose, 128, &[], 1).pixels
        );
    }
}
