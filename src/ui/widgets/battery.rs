use crate::core::{Detail, PowerSource};
use crate::ui::theme::{Role, color, label, size};
use crate::ui::{
    icons::Icon,
    motion::{self, Channel, tokens},
    strings as s,
};
use eframe::egui::{Rect, Sense, Ui, pos2, vec2};

pub fn show(ui: &mut Ui, details: &[Detail]) {
    let percent = details.iter().find_map(|d| {
        if let Detail::Battery(p) = d {
            Some(*p)
        } else {
            None
        }
    });
    let source = details.iter().find_map(|d| {
        if let Detail::Power(p) = d {
            Some(*p)
        } else {
            None
        }
    });
    let ink = if percent.is_some_and(|p| p <= 15) {
        color::STATUS_CAUTION.get()
    } else {
        color::TEXT_SECONDARY.get()
    };
    let text = s::battery_status(percent, source);
    ui.horizontal(|ui| {
        let (rect, response) =
            ui.allocate_exact_size(vec2(size::ICON_SMALL, size::ICON_SMALL), Sense::hover());
        let fraction = motion::animate(
            ui.ctx(),
            ui.id().with("battery-fill"),
            f32::from(percent.unwrap_or(0)) / 100.0,
            tokens::STANDARD,
            Channel::Fade,
        )
        .clamp(0.0, 1.0);
        let scale = rect.width() / 24.0;
        if fraction > 0.0 {
            let fill = Rect::from_min_size(
                rect.min + vec2(4.0, 8.0) * scale,
                vec2(13.0 * fraction, 8.0) * scale,
            );
            ui.painter().rect_filled(fill, scale, ink);
        }
        Icon::Battery.paint(ui.painter(), rect, ink);
        if source == Some(PowerSource::External) {
            let badge = Rect::from_min_size(
                pos2(rect.right() - 7.0, rect.bottom() - 7.0),
                vec2(9.0, 9.0),
            );
            ui.painter()
                .rect_filled(badge, 1.0, color::SURFACE_BASE.get());
            Icon::Plug.paint(ui.painter(), badge, ink);
        }
        response.on_hover_text(&text);
        label(ui, &text, Role::Caption, ink);
    });
}
