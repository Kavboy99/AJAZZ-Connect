//! A small depth-buffered triangle rasterizer. It draws a device model into an image that the
//! interface shows as an ordinary texture, so the 3D view needs no GPU code (and no `unsafe`),
//! and `--review` screenshots show exactly what the window shows.

use super::glb::{Image, Scene};
use super::light::{Lit, Rig, to_linear, to_srgb};
use super::math::{self, Mat4, Vec3};
use eframe::egui::ColorImage;
use std::sync::OnceLock;

const FIELD_OF_VIEW: f32 = 0.42;
/// Distance at which a unit sphere just fits, times a margin for the lift and the tilt.
const FIT_MARGIN: f32 = 1.1;
/// How high a fully lifted model floats, in model radii.
const LIFT: f32 = 0.06;
const SRGB_STEPS: usize = 4096;

/// While the pose moves, a half-resolution image keeps the frame cheap (the texture filter and
/// the motion hide it); at rest the view is drawn once more with 2×2 samples per pixel so edges and
/// key legends are smooth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quality {
    Draft,
    Fine,
}

impl Quality {
    /// Output side and samples per output pixel along each axis.
    fn layout(self, side: usize) -> (usize, usize) {
        match self {
            Self::Draft => (side.div_ceil(2), 1),
            Self::Fine => (side, 2),
        }
    }
}

/// How the model is held. Angles in radians; `lift` and `glow` in 0..1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
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
    uv: [f32; 2],
    lit: Lit,
}

struct Camera {
    model: Mat4,
    /// Model to clip space in one matrix.
    mvp: Mat4,
    half: f32,
}

impl Camera {
    fn new(scene: &Scene, pose: &Pose, side: usize) -> Self {
        let distance = FIT_MARGIN / (FIELD_OF_VIEW / 2.0).sin();
        let normalize = math::mul(
            &math::scale(1.0 / scene.radius),
            &math::translation([-scene.center[0], -scene.center[1], -scene.center[2]]),
        );
        let turn = math::mul(&math::rotation_x(pose.pitch), &math::rotation_y(pose.yaw));
        let rise = math::translation([0.0, pose.lift * LIFT, 0.0]);
        let model = math::mul(&rise, &math::mul(&turn, &normalize));
        let clip = math::mul(
            &math::perspective(FIELD_OF_VIEW, 1.0, distance - 2.0, distance + 2.0),
            &math::translation([0.0, 0.0, -distance]),
        );
        Self {
            mvp: math::mul(&clip, &model),
            model,
            half: side as f32 / 2.0,
        }
    }

    fn project(&self, position: Vec3) -> [f32; 3] {
        let p = math::transform_point(&self.mvp, position);
        [(p[0] + 1.0) * self.half, (1.0 - p[1]) * self.half, p[2]]
    }
}

fn sample(image: &Image, linear: &[f32; 256], uv: [f32; 2]) -> [f32; 4] {
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
    std::array::from_fn(|c| {
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
    })
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

struct Target {
    side: usize,
    /// Straight linear color and coverage.
    color: Vec<[f32; 4]>,
    depth: Vec<f32>,
}

impl Target {
    fn triangle(&mut self, [a, b, c]: [&Projected; 3], fill: &Fill<'_>) {
        let area = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
        // Screen Y points down, so front faces (counter-clockwise in glTF) have negative area.
        if area >= -f32::EPSILON {
            return;
        }
        let limit = (self.side - 1) as f32;
        let span = |lo: f32, hi: f32| {
            (
                lo.floor().clamp(0.0, limit) as usize,
                hi.ceil().clamp(0.0, limit) as usize,
            )
        };
        let (x0, x1) = span(a.x.min(b.x).min(c.x), a.x.max(b.x).max(c.x));
        let (y0, y1) = span(a.y.min(b.y).min(c.y), a.y.max(b.y).max(c.y));
        let (ea, eb) = (Edge::new(b, c, area), Edge::new(c, a, area));
        let linear = &tables().linear;
        for y in y0..=y1 {
            let py = y as f32 + 0.5;
            let row = y * self.side;
            for x in x0..=x1 {
                let px = x as f32 + 0.5;
                let (wa, wb) = (ea.at(px, py), eb.at(px, py));
                let wc = 1.0 - wa - wb;
                if wa < 0.0 || wb < 0.0 || wc < 0.0 {
                    continue;
                }
                let z = a.z * wa + b.z * wb + c.z * wc;
                let at = row + x;
                if z >= self.depth[at] {
                    continue;
                }
                let texel = fill.texture.map_or([1.0; 4], |image| {
                    let uv = std::array::from_fn(|i| a.uv[i] * wa + b.uv[i] * wb + c.uv[i] * wc);
                    sample(image, linear, uv)
                });
                let alpha = texel[3] * fill.alpha;
                if alpha <= 0.0 {
                    continue;
                }
                let rgb: [f32; 3] = std::array::from_fn(|i| {
                    let multiply =
                        a.lit.multiply[i] * wa + b.lit.multiply[i] * wb + c.lit.multiply[i] * wc;
                    let add = a.lit.add[i] * wa + b.lit.add[i] * wb + c.lit.add[i] * wc;
                    texel[i] * multiply + add
                });
                let out = &mut self.color[at];
                if fill.blend {
                    for i in 0..3 {
                        out[i] = rgb[i] * alpha + out[i] * (1.0 - alpha);
                    }
                    out[3] = alpha + out[3] * (1.0 - alpha);
                } else {
                    *out = [rgb[0], rgb[1], rgb[2], 1.0];
                    self.depth[at] = z;
                }
            }
        }
    }

    /// Box-filters `n`×`n` samples into each output pixel, in egui's premultiplied sRGB.
    fn resolve(&self, side: usize, n: usize) -> ColorImage {
        let srgb = &tables().srgb;
        let count = (n * n) as f32;
        let byte =
            |linear: f32| srgb[(linear.clamp(0.0, 1.0) * (SRGB_STEPS - 1) as f32).round() as usize];
        let mut pixels = Vec::with_capacity(side * side * 4);
        for y in 0..side {
            for x in 0..side {
                let mut sum = [0.0; 4];
                for sy in 0..n {
                    let row = (y * n + sy) * self.side + x * n;
                    for s in &self.color[row..row + n] {
                        for i in 0..3 {
                            sum[i] += s[i] * s[3];
                        }
                        sum[3] += s[3];
                    }
                }
                let alpha = sum[3] / count;
                for value in &sum[..3] {
                    let straight = if sum[3] > 0.0 { value / sum[3] } else { 0.0 };
                    pixels.push((f32::from(byte(straight)) * alpha).round() as u8);
                }
                pixels.push((alpha * 255.0).round() as u8);
            }
        }
        ColorImage::from_rgba_premultiplied([side, side], &pixels)
    }
}

/// The model in a square of `side` pixels (half of it for a draft) on a transparent background.
pub fn render(scene: &Scene, pose: &Pose, side: usize, quality: Quality) -> ColorImage {
    let (side, n) = quality.layout(side);
    let samples = side * n;
    let camera = Camera::new(scene, pose, samples);
    let rig = Rig::new(pose.glow_color.map(|c| c * pose.glow));
    let mut target = Target {
        side: samples,
        color: vec![[0.0; 4]; samples * samples],
        depth: vec![f32::INFINITY; samples * samples],
    };
    for batch in &scene.batches {
        let m = &batch.material;
        let surface = rig.surface(m);
        let projected: Vec<Projected> = batch
            .vertices
            .iter()
            .map(|v| {
                let [x, y, z] = camera.project(v.position);
                let normal = math::transform_direction(&camera.model, v.normal);
                Projected {
                    x,
                    y,
                    z,
                    uv: v.uv,
                    lit: rig.shade(normal, &surface),
                }
            })
            .collect();
        let fill = Fill {
            texture: m.texture.and_then(|t| scene.images.get(t)),
            alpha: m.color[3],
            blend: m.blend,
        };
        for t in batch.indices.as_chunks::<3>().0 {
            let corner = |i: u32| &projected[i as usize];
            target.triangle([corner(t[0]), corner(t[1]), corner(t[2])], &fill);
        }
    }
    target.resolve(side, n)
}
