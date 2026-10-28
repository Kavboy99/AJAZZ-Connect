//! Anti-aliased stroking of icon outlines into RGBA pixels, for the window and EXE icons. Free of
//! egui: `build.rs` includes this file.

use super::path::{Point, Polyline};

/// Premultiplied RGBA in 0..1, row-major, top-down.
pub struct Canvas {
    size: usize,
    pixels: Vec<[f32; 4]>,
}

fn premultiplied(color: [u8; 4], coverage: f32) -> [f32; 4] {
    let a = color[3] as f32 / 255.0 * coverage.clamp(0.0, 1.0);
    let c = |i: usize| color[i] as f32 / 255.0 * a;
    [c(0), c(1), c(2), a]
}

fn distance_to_segment(p: Point, a: Point, b: Point) -> f32 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let length = dx * dx + dy * dy;
    let t = if length == 0.0 {
        0.0
    } else {
        (((p.x - a.x) * dx + (p.y - a.y) * dy) / length).clamp(0.0, 1.0)
    };
    let (x, y) = (a.x + t * dx - p.x, a.y + t * dy - p.y);
    (x * x + y * y).sqrt()
}

impl Canvas {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            pixels: vec![[0.0; 4]; size * size],
        }
    }

    fn blend(&mut self, x: usize, y: usize, color: [u8; 4], coverage: f32) {
        if coverage <= 0.0 {
            return;
        }
        let src = premultiplied(color, coverage);
        let dst = &mut self.pixels[y * self.size + x];
        for c in 0..4 {
            dst[c] = src[c] + dst[c] * (1.0 - src[3]);
        }
    }

    /// The whole canvas as a square with rounded corners of `radius` pixels.
    pub fn fill_rounded_square(&mut self, radius: f32, color: [u8; 4]) {
        let half = self.size as f32 / 2.0;
        for y in 0..self.size {
            for x in 0..self.size {
                let qx = ((x as f32 + 0.5 - half).abs() - (half - radius)).max(0.0);
                let qy = ((y as f32 + 0.5 - half).abs() - (half - radius)).max(0.0);
                let d = (qx * qx + qy * qy).sqrt() - radius;
                self.blend(x, y, color, 0.5 - d);
            }
        }
    }

    /// Strokes `lines` (24-unit grid) placed at `origin` and scaled by `scale`, with round caps.
    pub fn stroke(
        &mut self,
        lines: &[Polyline],
        origin: Point,
        scale: f32,
        width: f32,
        color: [u8; 4],
    ) {
        let segments: Vec<(Point, Point)> = lines
            .iter()
            .flat_map(|line| {
                let map = |p: &Point| Point {
                    x: origin.x + p.x * scale,
                    y: origin.y + p.y * scale,
                };
                let points: Vec<Point> = line.points.iter().map(map).collect();
                let mut pairs: Vec<(Point, Point)> =
                    points.windows(2).map(|w| (w[0], w[1])).collect();
                if line.closed
                    && let (Some(first), Some(last)) = (points.first(), points.last())
                {
                    pairs.push((*last, *first));
                }
                pairs
            })
            .collect();
        for y in 0..self.size {
            for x in 0..self.size {
                let p = Point {
                    x: x as f32 + 0.5,
                    y: y as f32 + 0.5,
                };
                let d = segments
                    .iter()
                    .map(|(a, b)| distance_to_segment(p, *a, *b))
                    .fold(f32::MAX, f32::min);
                self.blend(x, y, color, width / 2.0 + 0.5 - d);
            }
        }
    }

    /// Straight (not premultiplied) RGBA bytes, top-down.
    pub fn into_rgba(self) -> Vec<u8> {
        self.pixels
            .iter()
            .flat_map(|p| {
                let a = p[3];
                let channel = |c: f32| {
                    let straight = if a > 0.0 { c / a } else { 0.0 };
                    (straight.clamp(0.0, 1.0) * 255.0).round() as u8
                };
                [
                    channel(p[0]),
                    channel(p[1]),
                    channel(p[2]),
                    (a.clamp(0.0, 1.0) * 255.0).round() as u8,
                ]
            })
            .collect()
    }
}
