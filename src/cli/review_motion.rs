//! Inspect actual model-image geometry on consecutive input frames.
use anyhow::{Result, ensure};
use eframe::egui::{Rect, Shape, epaint::ClippedShape};
use std::path::Path;

#[derive(Default)]
pub(super) struct Trace {
    frames: Vec<[f32; 4]>,
}

fn model_rect(shape: &Shape) -> Option<Rect> {
    match shape {
        Shape::Mesh(_) => {
            let r = shape.visual_bounding_rect();
            (r.width() > 100.0 && (r.width() - r.height()).abs() < 0.1).then_some(r)
        }
        Shape::Vec(shapes) => shapes
            .iter()
            .filter_map(model_rect)
            .max_by(|a, b| a.area().total_cmp(&b.area())),
        _ => None,
    }
}

impl Trace {
    pub fn capture(&mut self, frame: usize, shapes: &[ClippedShape]) {
        if let Some(r) = shapes
            .iter()
            .filter_map(|s| model_rect(&s.shape))
            .max_by(|a, b| a.area().total_cmp(&b.area()))
        {
            self.frames
                .push([frame as f32, r.width(), r.center().x, r.center().y]);
        }
    }

    pub fn verify(&self, out: &Path, name: &str, width: &str) -> Result<()> {
        std::fs::write(
            out.join(format!("{name}-{width}-framing.json")),
            serde_json::to_vec_pretty(&self.frames)?,
        )?;
        let end = self
            .frames
            .iter()
            .find(|v| v[0] == 139.0)
            .ok_or_else(|| anyhow::anyhow!("{name}: final model framing missing"))?;
        let moving: Vec<_> = self
            .frames
            .iter()
            .filter(|v| v[0] >= 25.0 && v[0] <= 139.0)
            .collect();
        ensure!(
            moving.len() == 115,
            "{name}: missing model frames during transition"
        );
        for pair in moving.windows(2) {
            for axis in 1..4 {
                let old = pair[0][axis];
                let value = pair[1][axis];
                ensure!(
                    value >= old.min(end[axis]) - 0.2 && value <= old.max(end[axis]) + 0.2,
                    "{name}: model framing reversed on frame {}, axis {axis}: {old} -> {value} -> {}",
                    pair[1][0],
                    end[axis]
                );
            }
        }
        Ok(())
    }
}
