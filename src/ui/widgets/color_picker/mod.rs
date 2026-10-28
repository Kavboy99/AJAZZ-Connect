//! Lighting color picker. The spectrum shows the user's data, so it is drawn as is (`docs/DESIGN.md`,
//! section 2, exception). Markers follow the pointer with light inertia and settle on
//! `spring.release`, keeping the gesture's velocity.

mod hsv;

use super::interaction::{Interaction, focus_ring, mix};
use crate::core::Rgb;
use crate::ui::motion::{self, tokens};
use crate::ui::theme::{color, radius, size, space};
use eframe::egui::{
    Color32, CornerRadius, Id, Mesh, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui,
    Vec2, WidgetInfo, WidgetType, pos2, vec2,
};
pub use hsv::Hsv;

const PLANE_HEIGHT: f32 = 160.0;
const HUE_HEIGHT: f32 = 16.0;
const GRID: usize = 12;
const MARKER: f32 = 8.0;
const SWATCH: f32 = 24.0;

/// Starting points for common lighting colors. Data, not styling.
const PRESETS: [Rgb; 8] = [
    Rgb::new(255, 255, 255),
    Rgb::new(255, 0, 0),
    Rgb::new(255, 128, 0),
    Rgb::new(255, 255, 0),
    Rgb::new(0, 255, 0),
    Rgb::new(0, 255, 255),
    Rgb::new(0, 0, 255),
    Rgb::new(255, 0, 255),
];

fn plane_mesh(rect: Rect, hue: f32) -> Mesh {
    let mut mesh = Mesh::default();
    for row in 0..=GRID {
        for col in 0..=GRID {
            let s = col as f32 / GRID as f32;
            let v = 1.0 - row as f32 / GRID as f32;
            let pos = pos2(
                rect.left() + s * rect.width(),
                rect.top() + (1.0 - v) * rect.height(),
            );
            mesh.colored_vertex(pos, Hsv { h: hue, s, v }.color32());
        }
    }
    let stride = (GRID + 1) as u32;
    for row in 0..GRID as u32 {
        for col in 0..GRID as u32 {
            let i = row * stride + col;
            mesh.add_triangle(i, i + 1, i + stride);
            mesh.add_triangle(i + 1, i + stride + 1, i + stride);
        }
    }
    mesh
}

fn hue_mesh(rect: Rect) -> Mesh {
    let mut mesh = Mesh::default();
    for i in 0..=6 {
        let t = i as f32 / 6.0;
        let c = Hsv {
            h: t,
            s: 1.0,
            v: 1.0,
        }
        .color32();
        let x = rect.left() + t * rect.width();
        mesh.colored_vertex(pos2(x, rect.top()), c);
        mesh.colored_vertex(pos2(x, rect.bottom()), c);
    }
    for i in 0..6u32 {
        let a = i * 2;
        mesh.add_triangle(a, a + 1, a + 2);
        mesh.add_triangle(a + 1, a + 3, a + 2);
    }
    mesh
}

fn frame(ui: &Ui, rect: Rect) {
    ui.painter().rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(size::STROKE, color::STROKE_DEFAULT.get()),
        StrokeKind::Outside,
    );
}

fn marker(ui: &Ui, center: Pos2, fill: Color32) {
    let painter = ui.painter();
    painter.circle_filled(center, MARKER, fill);
    painter.circle_stroke(
        center,
        MARKER,
        Stroke::new(size::FOCUS_RING, color::FOCUS_RING.get()),
    );
    painter.circle_stroke(
        center,
        MARKER + size::FOCUS_RING,
        Stroke::new(size::STROKE, color::SURFACE_SUNKEN.get()),
    );
}

/// Pointer position as a 0..1 fraction of `rect` while the button is held on `response`.
fn drag_fraction(response: &Response, rect: Rect) -> Option<Vec2> {
    let pointer = response.interact_pointer_pos()?;
    response.is_pointer_button_down_on().then(|| {
        vec2(
            ((pointer.x - rect.left()) / rect.width()).clamp(0.0, 1.0),
            ((pointer.y - rect.top()) / rect.height()).clamp(0.0, 1.0),
        )
    })
}

/// Shows the picker and returns true when the user changed the color.
pub fn color_picker(ui: &mut Ui, id: Id, hsv: &mut Hsv) -> bool {
    let before = *hsv;
    let width = ui.available_width().min(320.0);

    let (plane, plane_response) =
        ui.allocate_exact_size(vec2(width, PLANE_HEIGHT), Sense::click_and_drag());
    plane_response
        .widget_info(|| WidgetInfo::labeled(WidgetType::ColorButton, true, "saturation and value"));
    if let Some(f) = drag_fraction(&plane_response, plane) {
        hsv.s = f.x;
        hsv.v = 1.0 - f.y;
    }
    ui.painter().add(Shape::mesh(plane_mesh(plane, hsv.h)));
    frame(ui, plane);
    let dragging = plane_response.is_pointer_button_down_on();
    let target = pos2(
        plane.left() + hsv.s * plane.width(),
        plane.top() + (1.0 - hsv.v) * plane.height(),
    );
    let spec = if dragging {
        tokens::FAST
    } else {
        tokens::RELEASE
    };
    let at = motion::animate_pos(ui.ctx(), id.with("sv"), target, spec);
    marker(ui, at, hsv.color32());

    ui.add_space(space::M - ui.spacing().item_spacing.y);
    let (hue, hue_response) =
        ui.allocate_exact_size(vec2(width, HUE_HEIGHT), Sense::click_and_drag());
    hue_response.widget_info(|| WidgetInfo::labeled(WidgetType::Slider, true, "hue"));
    if let Some(f) = drag_fraction(&hue_response, hue) {
        hsv.h = f.x.min(0.999);
    }
    ui.painter().add(Shape::mesh(hue_mesh(hue)));
    frame(ui, hue);
    let spec = if hue_response.is_pointer_button_down_on() {
        tokens::FAST
    } else {
        tokens::RELEASE
    };
    let x = motion::animate_pos(
        ui.ctx(),
        id.with("hue"),
        pos2(hue.left() + hsv.h * hue.width(), hue.center().y),
        spec,
    );
    marker(
        ui,
        x,
        Hsv {
            h: hsv.h,
            s: 1.0,
            v: 1.0,
        }
        .color32(),
    );

    ui.add_space(space::M - ui.spacing().item_spacing.y);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(space::S);
        for preset in PRESETS {
            if swatch(ui, id.with(preset.hex()), preset, hsv.to_rgb() == preset).clicked() {
                *hsv = Hsv::from_rgb(preset, hsv.h);
            }
        }
    });
    *hsv != before
}

fn swatch(ui: &mut Ui, id: Id, rgb: Rgb, selected: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(SWATCH), Sense::click());
    let response = response.on_hover_text(rgb.hex());
    response
        .widget_info(|| WidgetInfo::selected(WidgetType::ColorButton, true, selected, rgb.hex()));
    let state = Interaction::of(ui, id, &response);
    let painter = ui.painter();
    let fill = Color32::from_rgb(rgb.r, rgb.g, rgb.b);
    let inset = rect.shrink(state.press * 1.0);
    painter.rect_filled(inset, CornerRadius::same(radius::CONTROL), fill);
    let ring = mix(
        color::STROKE_DEFAULT.get(),
        color::TEXT_PRIMARY.get(),
        if selected { 1.0 } else { state.hover },
    );
    painter.rect_stroke(
        inset,
        CornerRadius::same(radius::CONTROL),
        Stroke::new(size::STROKE, ring),
        StrokeKind::Outside,
    );
    focus_ring(painter, rect, radius::CONTROL, state.focus);
    response
}
