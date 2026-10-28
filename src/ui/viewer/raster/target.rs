use super::{Edge, Fill, Projected, SRGB_STEPS, sample, tables};
use eframe::egui::Color32;
pub(super) struct Target {
    y_start: usize,
    rows: usize,
    side: usize,
    /// Straight linear color and coverage.
    color: Vec<[f32; 4]>,
    depth: Vec<f32>,
    glow: Vec<[f32; 3]>,
}

impl Target {
    pub fn new(side: usize, y_start: usize, rows: usize, emission: bool) -> Self {
        Self {
            side,
            y_start,
            rows,
            color: vec![[0.0; 4]; side * rows],
            depth: vec![f32::INFINITY; side * rows],
            glow: if emission {
                vec![[0.0; 3]; side * rows]
            } else {
                Vec::new()
            },
        }
    }
    pub fn triangle(&mut self, [a, b, c]: [&Projected; 3], fill: &Fill<'_>) {
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
        let y0 = y0.max(self.y_start);
        let y1 = y1.min(self.y_start + self.rows - 1);
        if y0 > y1 {
            return;
        }
        let (ea, eb) = (Edge::new(b, c, area), Edge::new(c, a, area));
        let linear = &tables().linear;
        for y in y0..=y1 {
            let py = y as f32 + 0.5;
            let row = (y - self.y_start) * self.side;
            for x in x0..=x1 {
                let px = x as f32 + 0.5;
                let (wa, wb) = (ea.at(px, py), eb.at(px, py));
                let wc = 1.0 - wa - wb;
                if wa < -f32::EPSILON || wb < -f32::EPSILON || wc < -f32::EPSILON {
                    continue;
                }
                let z = a.z * wa + b.z * wb + c.z * wc;
                let at = row + x;
                if z >= self.depth[at] {
                    continue;
                }
                let denom = wa * a.inv_w + wb * b.inv_w + wc * c.inv_w;
                let (wa, wb, wc) = (
                    wa * a.inv_w / denom,
                    wb * b.inv_w / denom,
                    wc * c.inv_w / denom,
                );
                let (texel, legend) = fill.texture.map_or(([1.0; 4], 0.0), |image| {
                    let uv = std::array::from_fn(|i| a.uv[i] * wa + b.uv[i] * wb + c.uv[i] * wc);
                    sample(image, linear, uv)
                });
                let alpha = texel[3] * fill.alpha;
                if alpha <= 0.0 {
                    continue;
                }
                let emitted: [f32; 3] = std::array::from_fn(|i| {
                    (a.lit.emission[i] * wa + b.lit.emission[i] * wb + c.lit.emission[i] * wc)
                        * legend
                });
                let lit_legend = if emitted.iter().any(|e| *e > 0.01) {
                    legend
                } else {
                    0.0
                };
                let direct: [f32; 3] = std::array::from_fn(|i| {
                    a.lit.glow[i] * wa + b.lit.glow[i] * wb + c.lit.glow[i] * wc + emitted[i]
                });
                let rgb: [f32; 3] = std::array::from_fn(|i| {
                    let multiply =
                        a.lit.multiply[i] * wa + b.lit.multiply[i] * wb + c.lit.multiply[i] * wc;
                    let add = a.lit.add[i] * wa + b.lit.add[i] * wb + c.lit.add[i] * wc;
                    texel[i] * multiply * (1.0 - lit_legend * 0.92) + add + emitted[i]
                });
                if let Some(glow) = self.glow.get_mut(at) {
                    for i in 0..3 {
                        glow[i] = direct[i] * alpha
                            + if fill.blend {
                                glow[i] * (1.0 - alpha)
                            } else {
                                0.0
                            };
                    }
                }
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
    pub fn resolve(&self, side: usize, n: usize) -> super::bloom::Band {
        let srgb = &tables().srgb;
        let count = (n * n) as f32;
        let byte =
            |linear: f32| srgb[(linear.clamp(0.0, 1.0) * (SRGB_STEPS - 1) as f32).round() as usize];
        let mut pixels = Vec::with_capacity(side * self.rows / n * 4);
        for y in 0..self.rows / n {
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
        let pixels = pixels
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| Color32::from_rgba_premultiplied(p[0], p[1], p[2], p[3]))
            .collect();
        let mut glow = Vec::new();
        if !self.glow.is_empty() {
            glow.reserve(side * self.rows / n);
            for y in 0..self.rows / n {
                for x in 0..side {
                    let mut sum = [0.0; 3];
                    for sy in 0..n {
                        for sx in 0..n {
                            let sample = self.glow[(y * n + sy) * self.side + x * n + sx];
                            for c in 0..3 {
                                sum[c] += sample[c] / count;
                            }
                        }
                    }
                    glow.push(sum);
                }
            }
        }
        super::bloom::Band { pixels, glow }
    }
}
