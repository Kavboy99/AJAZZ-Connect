//! A small software rasterizer for egui meshes, so `--review` can save PNG screenshots without a
//! window or a GPU. Blending matches egui's premultiplied gamma-space output.

use eframe::egui::{self, Rect, TextureId, epaint::Primitive};
use std::collections::BTreeMap;

pub struct Texture {
    width: usize,
    height: usize,
    pixels: Vec<[f32; 4]>,
}

impl Texture {
    fn sample(&self, u: f32, v: f32) -> [f32; 4] {
        let x = (u * self.width as f32 - 0.5).clamp(0.0, (self.width - 1) as f32);
        let y = (v * self.height as f32 - 0.5).clamp(0.0, (self.height - 1) as f32);
        let (x0, y0) = (x.floor() as usize, y.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(self.width - 1), (y0 + 1).min(self.height - 1));
        let (fx, fy) = (x - x0 as f32, y - y0 as f32);
        let at = |x: usize, y: usize| self.pixels[y * self.width + x];
        let mut out = [0.0; 4];
        for (c, value) in out.iter_mut().enumerate() {
            let top = at(x0, y0)[c] * (1.0 - fx) + at(x1, y0)[c] * fx;
            let bottom = at(x0, y1)[c] * (1.0 - fx) + at(x1, y1)[c] * fx;
            *value = top * (1.0 - fy) + bottom * fy;
        }
        out
    }
}

/// Applies a frame's texture updates (full images and patches).
pub fn update_textures(textures: &mut BTreeMap<TextureId, Texture>, delta: &egui::TexturesDelta) {
    for (id, change) in &delta.set {
        let egui::ImageData::Color(image) = &change.image;
        let pixels: Vec<[f32; 4]> = image
            .pixels
            .iter()
            .map(|p| p.to_array().map(|c| c as f32 / 255.0))
            .collect();
        let [w, h] = image.size;
        match (change.pos, textures.get_mut(id)) {
            (Some([x, y]), Some(texture)) => {
                for row in 0..h {
                    for col in 0..w {
                        texture.pixels[(y + row) * texture.width + x + col] = pixels[row * w + col];
                    }
                }
            }
            _ => {
                textures.insert(
                    *id,
                    Texture {
                        width: w,
                        height: h,
                        pixels,
                    },
                );
            }
        }
    }
}

pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<[f32; 4]>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![[0.0, 0.0, 0.0, 1.0]; width * height],
        }
    }

    pub fn draw(
        &mut self,
        primitives: &[egui::ClippedPrimitive],
        textures: &BTreeMap<TextureId, Texture>,
    ) {
        for primitive in primitives {
            let Primitive::Mesh(mesh) = &primitive.primitive else {
                continue;
            };
            let Some(texture) = textures.get(&mesh.texture_id) else {
                continue;
            };
            for tri in mesh.indices.as_chunks::<3>().0 {
                let v = tri.map(|i| mesh.vertices[i as usize]);
                self.triangle(primitive.clip_rect, &v, texture);
            }
        }
    }

    fn triangle(&mut self, clip: Rect, v: &[egui::epaint::Vertex; 3], texture: &Texture) {
        let area = (v[1].pos - v[0].pos).x * (v[2].pos - v[0].pos).y
            - (v[1].pos - v[0].pos).y * (v[2].pos - v[0].pos).x;
        let finite = v.iter().all(|p| p.pos.x.abs() < 1e5 && p.pos.y.abs() < 1e5);
        if area.abs() < 1e-6 || !finite {
            return;
        }
        let bounds = Rect::from_points(&v.map(|p| p.pos)).intersect(clip);
        let x0 = bounds.left().floor().max(0.0) as usize;
        let x1 = bounds.right().ceil().max(0.0) as usize;
        let y0 = bounds.top().floor().max(0.0) as usize;
        let y1 = bounds.bottom().ceil().max(0.0) as usize;
        let edge = |a: egui::Pos2, b: egui::Pos2, x: f32, y: f32| {
            (b.x - a.x) * (y - a.y) - (b.y - a.y) * (x - a.x)
        };
        for y in y0..y1.min(self.height) {
            for x in x0..x1.min(self.width) {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                if !clip.contains(egui::pos2(px, py)) {
                    continue;
                }
                let w0 = edge(v[1].pos, v[2].pos, px, py) / area;
                let w1 = edge(v[2].pos, v[0].pos, px, py) / area;
                let w2 = 1.0 - w0 - w1;
                if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                    continue;
                }
                let u = w0 * v[0].uv.x + w1 * v[1].uv.x + w2 * v[2].uv.x;
                let t = w0 * v[0].uv.y + w1 * v[1].uv.y + w2 * v[2].uv.y;
                let tex = texture.sample(u, t);
                let colors = v.map(|p| p.color.to_array().map(|c| c as f32 / 255.0));
                let dst = &mut self.pixels[y * self.width + x];
                let mut src = [0.0; 4];
                for c in 0..4 {
                    src[c] = (w0 * colors[0][c] + w1 * colors[1][c] + w2 * colors[2][c]) * tex[c];
                }
                for c in 0..4 {
                    dst[c] = src[c] + dst[c] * (1.0 - src[3]);
                }
            }
        }
    }

    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let bytes: Vec<u8> = self
            .pixels
            .iter()
            .flat_map(|p| [p[0], p[1], p[2]].map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8))
            .collect();
        let image = image::RgbImage::from_raw(self.width as u32, self.height as u32, bytes)
            .ok_or_else(|| anyhow::anyhow!("image size"))?;
        image.save(path)?;
        Ok(())
    }
}
