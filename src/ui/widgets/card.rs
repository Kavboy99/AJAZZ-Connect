use crate::ui::theme::{color, radius, size};
use eframe::egui::{CornerRadius, Frame, Margin, Stroke, Ui};

fn frame() -> Frame {
    Frame::new()
        .fill(color::SURFACE_CARD.get())
        .stroke(Stroke::new(size::STROKE, color::STROKE_SUBTLE.get()))
        .corner_radius(CornerRadius::same(radius::OVERLAY))
        .inner_margin(Margin::same(size::CARD_PADDING as i8))
}

/// A static group: `surface.card`, radius 8, subtle border, no shadow.
pub fn card<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    frame()
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}
