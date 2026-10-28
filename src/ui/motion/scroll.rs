//! Kinetic scrolling: each wheel or touchpad delta is an impulse whose travel equals the delta,
//! spread over an exponential decay. No rubber-band edge: velocity stops at the ends.

use super::{dt, keep_alive, reduced, tokens};
use eframe::egui::{Id, ScrollArea, Ui, scroll_area::ScrollSource};

#[derive(Clone, Copy, Debug, Default)]
pub struct KineticScroll {
    offset: f32,
    velocity: f32,
    max: f32,
}

impl KineticScroll {
    pub fn show<R>(&mut self, ui: &mut Ui, id: Id, add: impl FnOnce(&mut Ui) -> R) -> R {
        let ctx = ui.ctx().clone();
        let hovered = ui.rect_contains_pointer(ui.max_rect());
        let delta = if hovered {
            ui.input(|i| i.raw_scroll_delta.y)
        } else {
            0.0
        };
        if delta != 0.0 {
            if reduced(&ctx) {
                self.offset -= delta;
                self.velocity = 0.0;
            } else {
                // ∫ v e^(−t/τ) dt = v τ, so this impulse travels exactly `delta`.
                self.velocity -= delta / tokens::SCROLL_DECAY;
            }
        }
        if self.velocity != 0.0 {
            let dt = dt(&ctx);
            self.offset +=
                self.velocity * tokens::SCROLL_DECAY * (1.0 - (-dt / tokens::SCROLL_DECAY).exp());
            self.velocity *= (-dt / tokens::SCROLL_DECAY).exp();
            if self.velocity.abs() < tokens::SCROLL_STOP {
                self.velocity = 0.0;
            }
        }
        if self.offset <= 0.0 || self.offset >= self.max {
            self.velocity = 0.0;
        }
        self.offset = self.offset.clamp(0.0, self.max);
        let output = ScrollArea::vertical()
            .id_salt(id)
            .auto_shrink([false, false])
            .scroll_source(ScrollSource {
                mouse_wheel: false,
                ..ScrollSource::ALL
            })
            .vertical_scroll_offset(self.offset)
            .show(ui, add);
        // The scroll bar may have been dragged: take the area's own offset back.
        self.offset = output.state.offset.y;
        self.max = (output.content_size.y - output.inner_rect.height()).max(0.0);
        if self.velocity != 0.0 {
            keep_alive(&ctx);
        }
        output.inner
    }
}
