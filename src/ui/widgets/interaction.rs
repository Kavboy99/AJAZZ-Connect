//! Spring-driven interaction state shared by every control: hover, press and keyboard focus each
//! ease between 0 and 1, and colors are mixed from tokens by those amounts.

use crate::ui::motion::{self, Channel, tokens};
use crate::ui::theme::{color, motion_tokens, size};
use eframe::egui::{Color32, CornerRadius, Id, Painter, Rect, Response, Stroke, StrokeKind, Ui};

#[derive(Clone, Copy, Debug)]
pub struct Interaction {
    pub hover: f32,
    pub press: f32,
    pub focus: f32,
    /// Render scale for press feedback.
    pub scale: f32,
}

impl Interaction {
    pub fn of(ui: &Ui, id: Id, response: &Response) -> Self {
        let ctx = ui.ctx();
        let enabled = response.enabled();
        let pressed = enabled && response.is_pointer_button_down_on();
        let hovered = enabled && (response.hovered() || pressed);
        let focused = response.has_focus() && ui.memory(|m| m.focused().is_some());
        let on = |b: bool| if b { 1.0 } else { 0.0 };
        Self {
            hover: motion::animate(
                ctx,
                id.with("hover"),
                on(hovered),
                tokens::FAST,
                Channel::Fade,
            ),
            press: motion::animate(
                ctx,
                id.with("press"),
                on(pressed),
                tokens::FAST,
                Channel::Fade,
            ),
            focus: motion::animate(
                ctx,
                id.with("focus"),
                on(focused),
                tokens::FAST,
                Channel::Fade,
            ),
            scale: motion::animate(
                ctx,
                id.with("scale"),
                if pressed {
                    motion_tokens::PRESS_SCALE
                } else {
                    1.0
                },
                tokens::FAST,
                Channel::Scale,
            ),
        }
    }

    /// Rest, hover and pressed tokens mixed by the current amounts.
    pub fn fill(&self, rest: Color32, hover: Color32, pressed: Color32) -> Color32 {
        mix(mix(rest, hover, self.hover), pressed, self.press)
    }
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    a.lerp_to_gamma(b, t.clamp(0.0, 1.0))
}

/// 2 px `focus.ring`, 2 px outside the control, faded by `opacity`.
pub fn focus_ring(painter: &Painter, rect: Rect, radius: u8, opacity: f32) {
    if opacity <= 0.0 {
        return;
    }
    let ring = rect.expand(size::FOCUS_OFFSET);
    painter.rect_stroke(
        ring,
        CornerRadius::same(radius + size::FOCUS_OFFSET as u8),
        Stroke::new(
            size::FOCUS_RING,
            color::FOCUS_RING.get().gamma_multiply(opacity),
        ),
        StrokeKind::Outside,
    );
}
