use super::{DevicePage, Mode, State};
use crate::ui::motion::{self, Channel, tokens};
use crate::ui::theme::{Role, color, size, space};
use crate::ui::{device_reveal, icons::Icon, strings as s, viewer};
use eframe::egui::{Id, Rect, Ui, UiBuilder, Vec2, pos2};
pub(super) fn draw(ui: &mut Ui, id: Id, page: &DevicePage<'_>, stage: Rect, state: &mut State) {
    let encoder = state.mode == Mode::Encoder;
    let editing = state.mode != Mode::Overview;
    let toward = ui
        .ctx()
        .pointer_hover_pos()
        .filter(|p| stage.contains(*p))
        .map_or(Vec2::ZERO, |p| {
            ((p - stage.center()) / (stage.size() * 0.5)).clamp(Vec2::splat(-1.0), Vec2::splat(1.0))
                * if editing { 0.28 } else { 1.0 }
        });
    let tilt_x = motion::animate(
        ui.ctx(),
        id.with("tilt-x"),
        toward.x,
        tokens::TILT,
        Channel::Scale,
    );
    let tilt_y = motion::animate(
        ui.ctx(),
        id.with("tilt-y"),
        toward.y,
        tokens::TILT,
        Channel::Scale,
    );
    let animate = |name, target| {
        motion::animate(
            ui.ctx(),
            id.with(name),
            target,
            tokens::DEVICE_REVEAL,
            Channel::Scale,
        )
    };
    let zoom = animate(
        "zoom",
        if encoder {
            2.2
        } else if editing {
            1.12
        } else {
            1.0
        },
    );
    let focus = viewer::binding(page.view.visual).map_or([0.0; 3], |b| b.encoder_focus());
    let focus = std::array::from_fn(|i| {
        animate(
            ["focus-x", "focus-y", "focus-z"][i],
            if encoder { focus[i] } else { 0.0 },
        )
    });
    let pose = viewer::Pose {
        yaw: animate("yaw", if editing { 0.0 } else { -0.24 })
            + tilt_x * crate::ui::theme::motion_tokens::TILT_YAW,
        pitch: animate(
            "pitch",
            if encoder {
                0.85
            } else if editing {
                0.88
            } else {
                0.9
            },
        ) + tilt_y * crate::ui::theme::motion_tokens::TILT_PITCH,
        zoom,
        focus,
        lift: 0.0,
        glow: f32::from(editing) * 0.4,
        glow_color: [
            color::DEVICE_GLOW.get().r(),
            color::DEVICE_GLOW.get().g(),
            color::DEVICE_GLOW.get().b(),
        ]
        .map(eframe::egui::ecolor::linear_f32_from_gamma_u8),
    };
    let preview = state.preview(ui, id);
    // A keyboard is wide: allow its transparent square to extend past the stage vertically.
    let aspect = viewer::binding(page.view.visual).map_or(1.0, |b| b.stage_aspect());
    let side = if encoder {
        stage.width()
    } else {
        stage.width().min(stage.height() * aspect)
    };
    let square = Rect::from_center_size(stage.center(), Vec2::splat(side));
    let mut model_ui = ui.new_child(UiBuilder::new().max_rect(stage));
    model_ui.set_clip_rect(stage.intersect(ui.clip_rect()));
    let shown = page.view.visual.map_or(viewer::Shown::Missing, |visual| {
        viewer::show_preview(
            &model_ui,
            device_reveal::tile_id(page.route).with("model"),
            square,
            visual,
            pose,
            &preview,
        )
    });
    if shown != viewer::Shown::Model {
        Icon::for_kind(page.view.kind).paint(
            model_ui.painter(),
            Rect::from_center_size(stage.center(), Vec2::splat(size::DEVICE_HERO_HEIGHT * 0.3)),
            color::TEXT_SECONDARY.get(),
        );
    }
    if editing {
        let hint = if encoder {
            s::STUDIO_ENCODER_HINT
        } else {
            s::STUDIO_KEYS
        };
        model_ui.painter().text(
            pos2(stage.center().x, stage.bottom() - space::L),
            eframe::egui::Align2::CENTER_BOTTOM,
            hint,
            Role::Caption.font(),
            color::TEXT_SECONDARY.get(),
        );
    }
}
