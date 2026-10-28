//! Hue-only LED colors: color selection never introduces a dim or black value.
use super::{Interaction, focus_ring};
use crate::core::Rgb;
use crate::ui::strings as s;
use crate::ui::theme::{Role, color, label, radius, size, space};
use eframe::egui::{
    Color32, Id, Key, Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetType, ecolor::Hsva, vec2,
};
use std::f32::consts::TAU;

pub struct Palette {
    hues: [f32; 3],
    active: usize,
}
impl Default for Palette {
    fn default() -> Self {
        Self {
            hues: [0.56, 0.82, 0.04],
            active: 0,
        }
    }
}
impl Palette {
    pub fn colors(&self, count: usize) -> Vec<Color32> {
        self.hues[..count.clamp(1, 3)]
            .iter()
            .map(|h| Hsva::new(*h, 1.0, 1.0, 1.0).into())
            .collect()
    }
    pub fn rgb(&self) -> Rgb {
        let c = self.colors(1)[0];
        Rgb::new(c.r(), c.g(), c.b())
    }
}

pub fn lighting_palette(
    ui: &mut Ui,
    id: Id,
    palette: &mut Palette,
    count: usize,
    max_side: f32,
) -> eframe::egui::Response {
    let count = count.clamp(1, 3);
    palette.active = palette.active.min(count - 1);
    let side = size::HUE_WHEEL.min(ui.available_width()).min(max_side);
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click_and_drag());
    response.widget_info(|| {
        WidgetInfo::labeled(WidgetType::Slider, ui.is_enabled(), s::LIGHTING_COLOR)
    });
    let center = rect.center();
    let outer = side * 0.5 - space::S;
    let thickness = size::HUE_RING.min(side * 0.08);
    let stride = thickness + space::S.min(side * 0.03);
    if (response.dragged() || response.clicked())
        && let Some(pos) = response.interact_pointer_pos()
    {
        let v = pos - center;
        if let Some(ring) = (0..count).find(|i| {
            let r = outer - *i as f32 * stride;
            v.length() >= r - thickness && v.length() <= r
        }) {
            palette.active = ring;
            palette.hues[ring] = (v.y.atan2(v.x) / TAU).rem_euclid(1.0);
        }
    }
    if response.has_focus() {
        let delta = ui.input(|i| {
            f32::from(i.key_pressed(Key::ArrowRight)) - f32::from(i.key_pressed(Key::ArrowLeft))
        });
        palette.hues[palette.active] =
            (palette.hues[palette.active] + delta / 72.0).rem_euclid(1.0);
        if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
            palette.active = (palette.active + 1) % count;
        }
        if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
            palette.active = (palette.active + count - 1) % count;
        }
    }
    super::hue_ring::show(
        ui,
        id.with("spectrum"),
        rect,
        count,
        outer,
        thickness,
        stride,
    );
    for ring in 0..count {
        let r = outer - ring as f32 * stride;
        let angle = palette.hues[ring] * TAU;
        let at = center + vec2(angle.cos(), angle.sin()) * (r - thickness * 0.5);
        let at = crate::ui::motion::animate_pos(
            ui.ctx(),
            id.with(("marker", ring)),
            at,
            crate::ui::motion::tokens::RELEASE,
        );
        let chosen: Color32 = Hsva::new(palette.hues[ring], 1.0, 1.0, 1.0).into();
        ui.painter().circle_filled(at, thickness * 0.38, chosen);
        ui.painter().circle_stroke(
            at,
            thickness * 0.38,
            Stroke::new(size::FOCUS_RING, color::SURFACE_SUNKEN.get()),
        );
        ui.painter().circle_stroke(
            at,
            thickness * 0.38 + size::FOCUS_RING,
            Stroke::new(size::STROKE, color::FOCUS_RING.get()),
        );
    }
    let r = (outer - count as f32 * stride - space::XS).max(space::S);
    let chosen: Color32 = Hsva::new(palette.hues[palette.active], 1.0, 1.0, 1.0).into();
    ui.painter().circle_filled(center, r, chosen);
    ui.painter().circle_stroke(
        center,
        r + space::XS,
        Stroke::new(size::FOCUS_RING, color::STROKE_CONTROL.get()),
    );
    let state = Interaction::of(ui, id, &response);
    focus_ring(ui.painter(), rect, radius::OVERLAY, state.focus);
    label(
        ui,
        s::STUDIO_HUE_HINT,
        Role::Caption,
        color::TEXT_SECONDARY.get(),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pointer_edits_only_the_ring_it_hits() {
        use eframe::egui::{
            CentralPanel, Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect,
        };
        let ctx = Context::default();
        let mut palette = Palette::default();
        let mut wheel = Rect::NOTHING;
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(320.0, 400.0));
        let _ = ctx.run(
            RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ctx| {
                CentralPanel::default().show(ctx, |ui| {
                    wheel = lighting_palette(
                        ui,
                        Id::new("test-wheel"),
                        &mut palette,
                        3,
                        size::HUE_WHEEL,
                    )
                    .rect;
                });
            },
        );
        let pos = wheel.center()
            + vec2(
                0.0,
                wheel.width() * 0.5 - space::S - size::HUE_RING - space::S - size::HUE_RING * 0.5,
            );
        for pressed in [true, false] {
            let _ = ctx.run(
                RawInput {
                    screen_rect: Some(screen),
                    events: vec![
                        Event::PointerMoved(pos),
                        Event::PointerButton {
                            pos,
                            button: PointerButton::Primary,
                            pressed,
                            modifiers: Modifiers::NONE,
                        },
                    ],
                    ..Default::default()
                },
                |ctx| {
                    CentralPanel::default().show(ctx, |ui| {
                        lighting_palette(
                            ui,
                            Id::new("test-wheel"),
                            &mut palette,
                            3,
                            size::HUE_WHEEL,
                        );
                    });
                },
            );
        }
        assert!((palette.hues[1] - 0.25).abs() < 0.001);
        assert_eq!(palette.hues[0], 0.56);
        assert_eq!(palette.hues[2], 0.04);
    }
    #[test]
    fn every_hue_stays_at_full_led_value() {
        for hue in 0..360 {
            let p = Palette {
                hues: [hue as f32 / 360.0; 3],
                active: 0,
            };
            for c in p.colors(3) {
                assert_eq!(c.r().max(c.g()).max(c.b()), 255);
            }
        }
    }
}
