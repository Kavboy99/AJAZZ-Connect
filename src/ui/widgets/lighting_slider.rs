//! Compact labeled lighting controls with a spring-driven, accent-outlined thumb.
use super::{Interaction, focus_ring};
use crate::ui::motion::{self, tokens};
use crate::ui::theme::{Role, color, label, radius, size, space};
use eframe::egui::{DragValue, Response, Slider, Stroke, Ui, UiBuilder, vec2};
use std::ops::RangeInclusive;

pub fn lighting_slider<T: eframe::egui::emath::Numeric>(
    ui: &mut Ui,
    text: &str,
    value: &mut T,
    range: RangeInclusive<T>,
    suffix: &str,
) -> Response {
    let min = range.start().to_f64();
    let max = range.end().to_f64();
    ui.scope_builder(
        UiBuilder::new().id(ui.make_persistent_id(("lighting-slider", text))),
        |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().interact_size.y = size::LIGHTING_ROW;
                ui.spacing_mut().button_padding.x = space::XS;
                ui.allocate_ui(vec2(size::LIGHTING_LABEL, size::LIGHTING_ROW), |ui| {
                    ui.set_min_width(size::LIGHTING_LABEL);
                    label(ui, text, Role::Caption, color::TEXT_PRIMARY.get());
                });
                let width = (ui.available_width() - size::LIGHTING_VALUE - space::S).max(space::L);
                let mut response = ui
                    .scope(|ui| {
                        ui.spacing_mut().interact_size.y = size::LIGHTING_ROW;
                        ui.spacing_mut().slider_width = width;
                        ui.spacing_mut().slider_rail_height = size::LIGHTING_RAIL;
                        ui.set_opacity(0.0);
                        ui.add(Slider::new(value, range.clone()).show_value(false))
                    })
                    .inner;
                let rail = eframe::egui::Rect::from_center_size(
                    response.rect.center(),
                    vec2(response.rect.width(), size::LIGHTING_RAIL),
                );
                ui.painter()
                    .rect_filled(rail, radius::CONTROL, color::STROKE_SUBTLE.get());
                let interaction = Interaction::of(ui, response.id, &response);
                let fraction =
                    ((value.to_f64() - min) / (max - min).max(f64::EPSILON)).clamp(0.0, 1.0) as f32;
                // Match egui's hit mapping while keeping the painted thumb smaller than its target.
                let track = response.rect.x_range().shrink(response.rect.height() / 2.5);
                let at = eframe::egui::pos2(
                    eframe::egui::lerp(track, fraction),
                    response.rect.center().y,
                );
                let at =
                    motion::animate_pos(ui.ctx(), response.id.with("thumb"), at, tokens::RELEASE);
                ui.painter().circle(
                    at,
                    size::LIGHTING_THUMB,
                    color::SLIDER_CENTER.get(),
                    Stroke::new(
                        size::FOCUS_RING,
                        interaction.fill(
                            color::ACCENT.get(),
                            color::ACCENT_HOVER.get(),
                            color::ACCENT_PRESSED.get(),
                        ),
                    ),
                );
                focus_ring(
                    ui.painter(),
                    response.rect,
                    radius::CONTROL,
                    interaction.focus,
                );
                let number = ui.add_sized(
                    vec2(size::LIGHTING_VALUE, size::LIGHTING_ROW),
                    DragValue::new(value)
                        .range(range)
                        .max_decimals(0)
                        .suffix(suffix),
                );
                if number.changed() {
                    response.mark_changed();
                }
                response.widget_info(|| {
                    eframe::egui::WidgetInfo::labeled(
                        eframe::egui::WidgetType::Slider,
                        ui.is_enabled(),
                        text,
                    )
                });
                response
            })
            .inner
        },
    )
    .inner
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{
        CentralPanel, Context, Event, Id, Modifiers, PointerButton, Pos2, RawInput, Rect,
    };

    #[test]
    fn row_ids_survive_preceding_widgets_and_brightness_does_not_edit_speed() {
        let ctx = Context::default();
        let mut brightness = 2_u8;
        let mut speed = 1_u8;
        let mut previous = [Id::NULL; 2];
        let mut target = Pos2::ZERO;
        for frame in 0..4 {
            let events = if frame >= 2 {
                vec![
                    Event::PointerMoved(target),
                    Event::PointerButton {
                        pos: target,
                        button: PointerButton::Primary,
                        pressed: frame == 2,
                        modifiers: Modifiers::NONE,
                    },
                ]
            } else {
                vec![]
            };
            let _ = ctx.run(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(360.0, 240.0))),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    CentralPanel::default().show(ctx, |ui| {
                        if frame > 0 {
                            ui.label("Changing preceding controls");
                        }
                        let a = lighting_slider(ui, "Brightness", &mut brightness, 0..=4, "");
                        let b = lighting_slider(ui, "Speed", &mut speed, 0..=4, "");
                        let ids = [a.id, b.id];
                        if frame > 0 {
                            assert_eq!(ids, previous);
                        }
                        previous = ids;
                        target = a.rect.right_center() - vec2(1.0, 0.0);
                    });
                },
            );
        }
        assert_eq!(brightness, 4);
        assert_eq!(speed, 1);
    }
}
