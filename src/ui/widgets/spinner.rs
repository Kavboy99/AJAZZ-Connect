//! Busy indicator: the one endless motion, drawn only while an operation runs.

use crate::ui::motion;
use crate::ui::theme::size;
use eframe::egui::{Color32, Rect, Shape, Stroke, Ui, Vec2, pos2};

/// Radians per second.
const SPEED: f32 = 5.0;
const SWEEP: f32 = 4.4;
const STEPS: usize = 24;

pub fn paint(ui: &Ui, rect: Rect, color: Color32) {
    let start = ui.input(|i| i.time) as f32 * SPEED;
    let r = rect.width().min(rect.height()) / 2.0 - size::STROKE;
    let c = rect.center();
    let points = (0..=STEPS)
        .map(|i| {
            let a = start + SWEEP * i as f32 / STEPS as f32;
            pos2(c.x + r * a.cos(), c.y + r * a.sin())
        })
        .collect();
    ui.painter()
        .add(Shape::line(points, Stroke::new(size::STROKE * 1.5, color)));
    motion::keep_alive(ui.ctx());
}

pub fn show(ui: &mut Ui, color: Color32) {
    let (rect, _) =
        ui.allocate_exact_size(Vec2::splat(size::ICON_SMALL), eframe::egui::Sense::hover());
    paint(ui, rect, color);
}
