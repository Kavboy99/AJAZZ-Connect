//! A square device tile (`docs/DESIGN.md`, section 8.1): the device model on a stage, its name
//! below. On hover the model rises, turns a little toward the pointer and the stage behind it
//! lights up; everything moves on springs and stops when the pointer leaves or rests.

use super::interaction::{Interaction, focus_ring};
use super::stage_spot as spot;
use crate::ui::icons::Icon;
use crate::ui::motion::{self, Channel, tokens};
use crate::ui::theme::{Role, color, motion_tokens, radius, rich, size, space};
use crate::ui::viewer::{self, Pose, Shown};
use eframe::egui::{
    Align, Color32, CornerRadius, Id, Label, Layout, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    UiBuilder, Vec2, pos2, vec2,
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

pub struct TileContent<'a> {
    pub name: &'a str,
    pub caption: &'a str,
    pub icon: Icon,
    pub visual: Option<&'a str>,
    pub busy: bool,
    /// Badges in the top left corner: icon + text, as everywhere.
    pub badges: &'a [(super::Tone, &'a str)],
}

/// Linear RGB of a token, for the rim light of the model.
fn linear(c: Color32) -> [f32; 3] {
    let f = |v: u8| eframe::egui::ecolor::linear_f32_from_gamma_u8(v);
    [f(c.r()), f(c.g()), f(c.b())]
}

pub fn device_tile(ui: &mut Ui, id: Id, side: f32, content: &TileContent<'_>) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click());
    let state = Interaction::of(ui, id, &response);
    let painter = ui.painter_at(rect);
    let corner = CornerRadius::same(radius::OVERLAY);
    let fill = state.fill(
        color::SURFACE_CARD.get(),
        color::SURFACE_CARD_HOVER.get(),
        color::SURFACE_CARD.get(),
    );
    painter.rect_filled(rect, corner, fill);
    if !crate::ui::device_reveal::hidden(ui.ctx(), id) {
        device_stage(ui, id, tile_stage(rect), content.visual, content.icon);
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

pub fn tile_stage(rect: Rect) -> Rect {
    Rect::from_min_max(rect.min, pos2(rect.max.x, rect.max.y - FOOTER))
}

/// Shared interactive stage for the tile and the expanded device page.
pub fn device_stage(ui: &mut Ui, id: Id, stage: Rect, visual: Option<&str>, icon: Icon) {
    let side = stage.width().min(stage.height() + FOOTER);
    let ctx = ui.ctx().clone();
    let hovered = ui.rect_contains_pointer(stage);
    // The pointer position over the tile, -1..1 on both axes; the model turns toward it.
    let toward = ui
        .ctx()
        .pointer_hover_pos()
        .filter(|_| hovered)
        .map_or(Vec2::ZERO, |p| {
            ((p - stage.center()) / (stage.size() / 2.0)).clamp(Vec2::splat(-1.0), Vec2::splat(1.0))
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

    let painter = ui.painter_at(ui.clip_rect());
    let rise = motion_tokens::TILE_RISE * lift;
    let center = stage.center() - vec2(0.0, rise);
    let glow = color::DEVICE_GLOW.get();
    spot(
        &painter,
        stage.center(),
        vec2(side * 0.5, side * 0.36),
        glow.gamma_multiply(GLOW_ALPHA * lift),
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
        studio: 0.0,
        zoom: 1.0,
        focus: [0.0; 3],
        yaw: BASE_YAW + tilt_x * motion_tokens::TILT_YAW,
        pitch: BASE_PITCH + tilt_y * motion_tokens::TILT_PITCH,
        lift,
        glow: lift,
        glow_color: linear(glow),
    };
    let model = Rect::from_center_size(center, Vec2::splat(side - 2.0 * size::CARD_PADDING));
    let shown = visual.map_or(Shown::Missing, |visual| {
        viewer::show(ui, id.with("model"), model, visual, pose, 1.0)
    });
    if shown != Shown::Model {
        let offset = vec2(tilt_x * space::S, tilt_y * space::XS);
        let icon_rect = Rect::from_center_size(center + offset, Vec2::splat(ICON_SIZE));
        icon.paint(&painter, icon_rect, color::TEXT_SECONDARY.get());
    }
}
