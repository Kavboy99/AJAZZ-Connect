use crate::ui::motion::{self, Channel, tokens};
use crate::ui::theme::{Role, color, radius, size, space};
use eframe::egui::{
    CornerRadius, Frame, Id, Margin, Rect, Response, Stroke, TextEdit, Ui, pos2, vec2,
};

/// Input field: `surface.sunken`, `stroke.control` border, a 2 px accent bottom line that fades in
/// on focus.
pub fn text_field(ui: &mut Ui, id: Id, text: &mut String, width: f32, mono: bool) -> Response {
    let font = if mono { Role::Mono } else { Role::Body }.font();
    let inner_width = width - space::S * 2.0;
    let inner = Frame::new()
        .fill(color::SURFACE_SUNKEN.get())
        .stroke(Stroke::new(size::STROKE, color::STROKE_CONTROL.get()))
        .corner_radius(CornerRadius::same(radius::CONTROL))
        .inner_margin(Margin::symmetric(space::S as i8, space::S as i8))
        .show(ui, |ui| {
            ui.add_sized(
                vec2(inner_width, size::CONTROL_HEIGHT - space::L),
                TextEdit::singleline(text)
                    .id(id)
                    .font(font)
                    .text_color(color::TEXT_PRIMARY.get())
                    .frame(false)
                    .margin(Margin::ZERO)
                    .desired_width(inner_width),
            )
        });
    let response = inner.inner;
    let rect = inner.response.rect;
    let focus = motion::animate(
        ui.ctx(),
        id.with("underline"),
        if response.has_focus() { 1.0 } else { 0.0 },
        tokens::FAST,
        Channel::Fade,
    );
    if focus > 0.0 {
        let line = Rect::from_min_max(
            pos2(rect.left() + size::STROKE, rect.bottom() - size::FOCUS_RING),
            pos2(rect.right() - size::STROKE, rect.bottom()),
        );
        ui.painter().rect_filled(
            line,
            CornerRadius::ZERO,
            color::ACCENT.get().gamma_multiply(focus),
        );
    }
    response
}
