//! A square device tile (`docs/DESIGN.md`, section 8.1): the device model on a stage, its name
//! below. On hover the model rises, turns a little toward the pointer and the stage behind it
//! lights up; everything moves on springs and stops when the pointer leaves or rests.

use super::interaction::{Interaction, focus_ring};
use crate::ui::icons::Icon;
use crate::ui::motion::{self, Channel, tokens};
use crate::ui::theme::{Role, color, motion_tokens, radius, rich, size, space};
use crate::ui::viewer::{self, Pose, Shown};
use eframe::egui::epaint::Mesh;
use eframe::egui::{
    Align, Color32, CornerRadius, Id, Label, Layout, Painter, Pos2, Rect, Response, Sense, Shape,
    Stroke, StrokeKind, Ui, UiBuilder, Vec2, pos2, vec2,
};

/// The resting view: from the front, above and a little to the left.
const BASE_YAW: f32 = -0.35;
const BASE_PITCH: f32 = 0.9;
/// Height of the name and caption under the model.
const FOOTER: f32 = 44.0;
/// The kind icon that stands in for a device without a model.
const ICON_SIZE: f32 = 64.0;
const GLOW_ALPHA: f32 = 0.30;
const SHADOW_ALPHA: f32 = 0.45;
const SEGMENTS: usize = 48;

pub struct TileContent<'a> {
    pub name: &'a str,
    pub caption: &'a str,
    pub icon: Icon,
    pub visual: Option<&'a str>,
    pub busy: bool,
    /// Badges in the top left corner: icon + text, as everywhere.
    pub badges: &'a [(super::Tone, &'a str)],
}

/// A soft radial spot: `color` in the middle, fading out at the ellipse `radii`.
fn spot(painter: &Painter, center: Pos2, radii: Vec2, color: Color32) {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(center, color);
    let rings = [
        (0.45, color.gamma_multiply(0.55)),
        (1.0, Color32::TRANSPARENT),
    ];
    for (scale, ring_color) in rings {
        for i in 0..SEGMENTS {
            let angle = i as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            let offset = vec2(angle.cos() * radii.x, angle.sin() * radii.y) * scale;
            mesh.colored_vertex(center + offset, ring_color);
        }
    }
    let n = SEGMENTS as u32;
    for i in 0..n {
        let j = (i + 1) % n;
        mesh.add_triangle(0, 1 + i, 1 + j);
        mesh.add_triangle(1 + i, 1 + n + i, 1 + n + j);
        mesh.add_triangle(1 + i, 1 + n + j, 1 + j);
    }
    painter.add(Shape::mesh(mesh));
}

/// Linear RGB of a token, for the rim light of the model.
fn linear(c: Color32) -> [f32; 3] {
    let f = |v: u8| eframe::egui::ecolor::linear_f32_from_gamma_u8(v);
    [f(c.r()), f(c.g()), f(c.b())]
}

pub fn device_tile(ui: &mut Ui, id: Id, side: f32, content: &TileContent<'_>) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click());
    let state = Interaction::of(ui, id, &response);
    let ctx = ui.ctx().clone();
    let hovered = response.hovered();
    // The pointer position over the tile, -1..1 on both axes; the model turns toward it.
    let toward = response
        .hover_pos()
        .filter(|_| hovered)
        .map_or(Vec2::ZERO, |p| {
            ((p - rect.center()) / (side / 2.0)).clamp(Vec2::splat(-1.0), Vec2::splat(1.0))
        });
    let turn = |axis: &str, target: f32| {
        motion::animate(&ctx, id.with(axis), target, tokens::TILT, Channel::Scale)
    };
    let (tilt_x, tilt_y) = (turn("tilt-x", toward.x), turn("tilt-y", toward.y));
    let lift = motion::animate(
        &ctx,
        id.with("lift"),
        if hovered { 1.0 } else { 0.0 },
        tokens::STANDARD,
        Channel::Scale,
    );

    let painter = ui.painter_at(rect);
    let corner = CornerRadius::same(radius::OVERLAY);
    let fill = state.fill(
        color::SURFACE_CARD.get(),
        color::SURFACE_CARD_HOVER.get(),
        color::SURFACE_CARD.get(),
    );
    painter.rect_filled(rect, corner, fill);

    let stage = Rect::from_min_max(rect.min, pos2(rect.max.x, rect.max.y - FOOTER));
    let rise = motion_tokens::TILE_RISE * lift;
    let center = stage.center() - vec2(0.0, rise);
    let glow = color::DEVICE_GLOW.get();
    spot(
        &painter,
        stage.center(),
        vec2(side * 0.5, side * 0.36),
        glow.gamma_multiply(GLOW_ALPHA * state.hover),
    );
    spot(
        &painter,
        stage.center() + vec2(0.0, side * 0.16),
        vec2(side * 0.36, side * 0.05) * (1.0 - 0.15 * lift),
        color::DEVICE_SHADOW
            .get()
            .gamma_multiply(SHADOW_ALPHA * (1.0 - 0.4 * lift)),
    );

    let pose = Pose {
        yaw: BASE_YAW + tilt_x * motion_tokens::TILT_YAW,
        pitch: BASE_PITCH + tilt_y * motion_tokens::TILT_PITCH,
        lift,
        glow: state.hover,
        glow_color: linear(glow),
    };
    let model = Rect::from_center_size(center, Vec2::splat(side - 2.0 * size::CARD_PADDING));
    let shown = content.visual.map_or(Shown::Missing, |visual| {
        viewer::show(ui, id.with("model"), model, visual, pose, 1.0)
    });
    if shown != Shown::Model {
        let offset = vec2(tilt_x * space::S, tilt_y * space::XS);
        let icon = Rect::from_center_size(center + offset, Vec2::splat(ICON_SIZE));
        content
            .icon
            .paint(&painter, icon, color::TEXT_SECONDARY.get());
    }

    let inner = rect.shrink(size::CARD_PADDING);
    let footer = Rect::from_min_max(pos2(inner.min.x, rect.max.y - FOOTER - space::S), inner.max);
    let mut text = ui.new_child(
        UiBuilder::new()
            .max_rect(footer)
            .layout(Layout::top_down(Align::Min)),
    );
    text.spacing_mut().item_spacing.y = 0.0;
    text.add(
        Label::new(rich(
            content.name,
            Role::BodyStrong,
            color::TEXT_PRIMARY.get(),
        ))
        .truncate(),
    );
    text.add(
        Label::new(rich(
            content.caption,
            Role::Caption,
            color::TEXT_SECONDARY.get(),
        ))
        .truncate(),
    );
    let mut top = ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::left_to_right(Align::Min)),
    );
    top.spacing_mut().item_spacing.x = space::S;
    for (tone, label) in content.badges {
        super::badge(&mut top, *tone, label);
    }
    if content.busy {
        super::spinner::show(&mut top, color::TEXT_SECONDARY.get());
    }

    painter.rect_stroke(
        rect,
        corner,
        Stroke::new(size::STROKE, color::STROKE_SUBTLE.get()),
        StrokeKind::Inside,
    );
    focus_ring(ui.painter(), rect, radius::OVERLAY, state.focus);
    response.on_hover_cursor(eframe::egui::CursorIcon::PointingHand)
}
