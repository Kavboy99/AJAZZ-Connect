//! Opt-in native-window metrics for the hardware-free developer preview.
use eframe::egui::Context;
use std::time::Instant;
use std::{path::PathBuf, time::Duration};
pub(super) struct Capture {
    ctx: Context,
    path: PathBuf,
    samples: Vec<f64>,
    frames: usize,
    started: Instant,
    idle: Option<serde_json::Value>,
}
impl Capture {
    pub fn new(ctx: Context, path: PathBuf) -> Self {
        Self {
            ctx,
            path,
            samples: Vec::new(),
            frames: 0,
            started: Instant::now(),
            idle: None,
        }
    }
    pub fn record(&mut self, elapsed: Duration) {
        if self.frames > 10 && self.idle.is_none() && !self.ctx.has_requested_repaint() {
            self.idle = Some(crate::ui::gpu_stats(&self.ctx));
        }
        let left = Duration::from_secs(15).saturating_sub(self.started.elapsed());
        if left.is_zero() {
            self.ctx
                .send_viewport_cmd(eframe::egui::ViewportCommand::Close);
        } else {
            self.ctx.request_repaint_after(left);
        }
        self.frames += 1;
        if self.frames > 10 && self.samples.len() < 4096 {
            self.samples.push(elapsed.as_secs_f64() * 1000.0);
        }
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        crate::ui::viewer::gpu::finish_frame(&self.ctx, true);
        let mut stats = crate::ui::gpu_stats(&self.ctx);
        stats["ui_frames"] = self.frames.into();
        stats["first_idle"] = self.idle.take().unwrap_or(serde_json::Value::Null);
        stats["ui_update_mean_ms"] =
            serde_json::json!(self.samples.iter().sum::<f64>() / self.samples.len().max(1) as f64);
        stats["ui_update_max_ms"] =
            serde_json::json!(self.samples.iter().copied().fold(0.0, f64::max));
        if let Ok(bytes) = serde_json::to_vec_pretty(&stats) {
            let _ = std::fs::write(&self.path, bytes);
        }
    }
}
