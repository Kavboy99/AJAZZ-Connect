use super::{DevicePage, Mode, Model, Route, State};
use crate::ui::theme::{Role, color, label, size, space};
use crate::ui::widgets::lighting_palette::{lighting_palette, saturation};
use crate::ui::{strings as s, viewer::Effect};
use eframe::egui::{Id, Ui};

pub(super) fn show(ui: &mut Ui, model: &mut Model<'_>, page: &DevicePage<'_>, state: &mut State) {
    if state.mode == Mode::Encoder {
        encoder(ui, state);
        return;
    }
    let before = state.palette.rgb();
    let compact = ui.available_height() < size::DEVICE_HERO_HEIGHT;
    let count = if state.programs.is_some() {
        1
    } else {
        state.effect.color_count()
    };
    let id = Id::new(("studio-picker", page.route));
    if compact {
        let side = (ui.available_width() - size::LIGHTING_CONTROLS - space::S).max(space::L);
        ui.horizontal_top(|ui| {
            lighting_palette(ui, id, &mut state.palette, count, side);
            ui.vertical(|ui| {
                saturation(ui, &mut state.palette);
                super::programs::controls(ui, state);
            });
        });
    } else {
        label(
            ui,
            state.choice_label(),
            Role::BodyStrong,
            color::TEXT_PRIMARY.get(),
        );
        let rows = state.programs.as_ref().map_or(1, |p| {
            1 + usize::from(p.preset().brightness_max.is_some())
                + usize::from(p.preset().speed_max.is_some())
        });
        let reserved =
            rows as f32 * (size::LIGHTING_ROW + space::S) + size::CONTROL_HEIGHT * 3.0 + space::L;
        let side = (ui.available_height() - reserved).clamp(space::L, size::HUE_WHEEL);
        lighting_palette(ui, id, &mut state.palette, count, side);
        saturation(ui, &mut state.palette);
        super::programs::controls(ui, state);
    }
    if let Some(p) = &mut state.programs
        && p.preset().custom_color
        && before != state.palette.rgb()
    {
        p.preset_mut().saved.color = Some(state.palette.rgb());
    }
    if before != state.palette.rgb() {
        state.replay();
    }
    ui.horizontal_wrapped(|ui| {
        if let Some(programs) = &state.programs {
            state
                .apply
                .button(ui, super::apply::Action::Apply, programs.settings());
        }
        if let Route::Device(slot) = page.route
            && let Some(traits) = page.traits
            && (state.effect == Effect::Solid || state.programs.is_some())
        {
            super::super::lighting::actions(
                ui,
                model,
                slot,
                traits,
                state.programs.is_none().then_some(state.palette.rgb()),
                &mut state.hardware,
            );
        }
    });
}

fn encoder(ui: &mut Ui, state: &mut State) {
    label(
        ui,
        s::STUDIO_ENCODER_STEP,
        Role::BodyStrong,
        color::TEXT_PRIMARY.get(),
    );
    ui.add(eframe::egui::Slider::new(&mut state.step, 1..=20).text(s::STUDIO_ENCODER_STEP));
    ui.add_space(space::L);
    label(
        ui,
        s::STUDIO_ENCODER_PRESS,
        Role::BodyStrong,
        color::TEXT_PRIMARY.get(),
    );
    let press = match state.encoder {
        0 => s::STUDIO_ENCODER_MUTE,
        1 => s::STUDIO_ENCODER_PAUSE,
        _ => s::STUDIO_ENCODER_DEFAULT,
    };
    ui.checkbox(&mut state.press, press);
    ui.add_space(space::L);
    label(
        ui,
        s::STUDIO_ENCODER_HINT,
        Role::Caption,
        color::TEXT_SECONDARY.get(),
    );
}
