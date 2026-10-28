use super::{DevicePage, Mode, Model, Route, State};
use crate::ui::theme::{Role, color, label, size, space};
use crate::ui::widgets::lighting_palette::lighting_palette;
use crate::ui::{strings as s, viewer::Effect};
use eframe::egui::{Id, Ui};

pub(super) fn show(ui: &mut Ui, model: &mut Model<'_>, page: &DevicePage<'_>, state: &mut State) {
    let compact = ui.available_height() < size::DEVICE_HERO_HEIGHT;
    if state.mode != Mode::Encoder {
        label(
            ui,
            state.choice_label(),
            Role::BodyStrong,
            color::TEXT_PRIMARY.get(),
        );
        let before = state.palette.rgb();
        let wheel_side =
            (ui.available_height() * 0.48).clamp(size::HUE_RING * 4.0, size::HUE_WHEEL);
        lighting_palette(
            ui,
            Id::new(("studio-picker", page.route)),
            &mut state.palette,
            if state.programs.is_some() {
                1
            } else {
                state.effect.color_count()
            },
            wheel_side,
        );
        if let Some(p) = &mut state.programs
            && p.preset().custom_color
        {
            p.preset_mut().saved.color = Some(state.palette.rgb());
        }
        if before != state.palette.rgb() {
            state.replay();
        }
        ui.add_space(space::S);
    }
    eframe::egui::ScrollArea::vertical()
        .id_salt("studio-inspector-scroll")
        .scroll_bar_visibility(eframe::egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .show(ui, |ui| {
            let title = if state.mode == Mode::Encoder {
                [s::STUDIO_VOLUME, s::STUDIO_SCROLL, s::STUDIO_BRIGHTNESS][state.encoder]
            } else {
                state.choice_label()
            };
            if !compact && state.mode == Mode::Encoder {
                label(ui, title, Role::Subtitle, color::TEXT_PRIMARY.get());
                ui.add_space(space::S);
                label(
                    ui,
                    if state.preview_requires_usb {
                        s::LIGHTING_PREVIEW_VALUES
                    } else if state.programs.is_some() {
                        s::LIGHTING_NATIVE_PREVIEW
                    } else {
                        s::STUDIO_PREVIEW
                    },
                    Role::Caption,
                    color::TEXT_SECONDARY.get(),
                );
                ui.add_space(space::L);
            }
            if state.mode == Mode::Encoder {
                encoder(ui, state);
            } else {
                super::programs::controls(ui, state);
                if !compact && state.mode == Mode::Encoder {
                    ui.add_space(space::M);
                }
                if let Route::Device(slot) = page.route
                    && let Some(traits) = page.traits
                {
                    if !compact && state.mode == Mode::Encoder {
                        ui.add_space(space::L);
                    }
                    if state.effect == Effect::Solid || state.programs.is_some() {
                        super::super::lighting::actions(
                            ui,
                            model,
                            slot,
                            traits,
                            state.palette.rgb(),
                            state.programs.as_ref().map(|p| p.settings()),
                            &mut state.hardware,
                        );
                    } else {
                        label(
                            ui,
                            s::STUDIO_EFFECT_PREVIEW,
                            Role::Caption,
                            color::TEXT_SECONDARY.get(),
                        );
                    }
                }
                if page.traits.is_none() && state.programs.is_some() {
                    ui.add(
                        crate::ui::widgets::Button::new(s::LIGHTING_APPLY_PROGRAM).enabled(false),
                    )
                    .on_hover_text(
                        if matches!(page.route, Route::Preview(_)) {
                            s::LIGHTING_OFFLINE_CONTROLS
                        } else {
                            s::LIGHTING_CONNECT_USB
                        },
                    );
                }
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
