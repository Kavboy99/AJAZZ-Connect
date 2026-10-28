use super::interaction::{Interaction, focus_ring, mix};
use crate::ui::motion::{self, Channel, tokens};
use crate::ui::theme::{Role, color, radius, rich, size, space};
use eframe::egui::{
    CornerRadius, Label, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo,
    WidgetType, vec2,
};

const TRACK: Vec2 = Vec2::new(40.0, 20.0);
const THUMB: f32 = 12.0;

/// A labeled on/off switch. The thumb travels and the fill flows on `spring.standard`.
pub fn switch(ui: &mut Ui, on: &mut bool, label: &str) -> Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::M;
        let (rect, mut response) =
            ui.allocate_exact_size(vec2(TRACK.x, size::CONTROL_HEIGHT), Sense::click());
        if response.clicked() {
            *on = !*on;
            response.mark_changed();
        }
        response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, true, *on, label));
        let state = Interaction::of(ui, response.id, &response);
        let t = motion::animate(
            ui.ctx(),
            response.id.with("on"),
            if *on { 1.0 } else { 0.0 },
            tokens::STANDARD,
            Channel::Fade,
        );
        let track = Rect::from_center_size(rect.center(), TRACK);
        let rest = mix(color::SURFACE_SUNKEN.get(), color::ACCENT.get(), t);
        let hover = mix(
            color::CONTROL_FILL_HOVER.get(),
            color::ACCENT_HOVER.get(),
            t,
        );
        let fill = state.fill(rest, hover, hover);
        let painter = ui.painter();
        painter.rect_filled(track, CornerRadius::same(radius::CONTROL), fill);
        let border = mix(color::STROKE_CONTROL.get(), color::ACCENT.get(), t);
        painter.rect_stroke(
            track,
            CornerRadius::same(radius::CONTROL),
            Stroke::new(size::STROKE, border),
            StrokeKind::Inside,
        );
        // The thumb stretches slightly while pressed.
        let stretch = THUMB * 0.25 * state.press;
        let travel = TRACK.x - THUMB - space::XS * 2.0;
        let x = track.left() + space::XS + travel * t;
        let thumb = Rect::from_min_size(
            eframe::egui::pos2(x - stretch * t, track.center().y - THUMB / 2.0),
            vec2(THUMB + stretch, THUMB),
        );
        let ink = mix(color::TEXT_SECONDARY.get(), color::TEXT_ON_ACCENT.get(), t);
        painter.rect_filled(thumb, CornerRadius::same(radius::CONTROL), ink);
        focus_ring(painter, track, radius::CONTROL, state.focus);
        ui.add(Label::new(rich(label, Role::Body, color::TEXT_PRIMARY.get())).wrap());
        response
    })
    .inner
}
