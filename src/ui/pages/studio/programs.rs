//! Native drafts and the independently observed program on the device.
use super::State;
use crate::core::{LightingEditor, LightingEffect, LightingPreset, LightingSettings};

use crate::ui::{strings as s, viewer::Effect};
use eframe::egui::Ui;

pub(super) struct Programs {
    source: LightingEditor,
    pub presets: Vec<LightingPreset>,
    pub selected: usize,
    confirmed: Option<u8>,
}

impl Programs {
    pub fn observe_device(&mut self, current: Option<u8>, failed: bool) {
        if !failed {
            self.confirmed = current;
        }
    }
    pub fn is_applied(&self, preset: u8) -> bool {
        self.confirmed == Some(preset)
    }
    pub fn settings(&self) -> LightingSettings {
        self.presets[self.selected].saved
    }
    pub fn preset(&self) -> &LightingPreset {
        &self.presets[self.selected]
    }
    pub fn preset_mut(&mut self) -> &mut LightingPreset {
        &mut self.presets[self.selected]
    }
}

pub(super) fn sync(state: &mut State, source: Option<&LightingEditor>) {
    let Some(source) = source else { return };
    if state.programs.as_ref().is_none_or(|p| p.source != *source) && !source.presets.is_empty() {
        let selected = source
            .presets
            .iter()
            .position(|p| p.saved.preset == source.current)
            .unwrap_or(0);
        if state.programs.is_some() {
            return;
        }
        state.programs = Some(Programs {
            source: source.clone(),
            presets: source.presets.clone(),
            selected,
            confirmed: None,
        });
        if let Some(p) = &mut state.programs {
            for preset in &mut p.presets {
                if preset.custom_color {
                    preset.saved.color = Some(state.palette.rgb());
                }
            }
        }
        update_preview(state);
    }
}

pub(super) fn update_preview(state: &mut State) {
    if let Some(p) = &state.programs {
        state.effect = match p.preset().effect {
            LightingEffect::Solid | LightingEffect::Off => Effect::Solid,
            LightingEffect::Reactive | LightingEffect::Ripples | LightingEffect::Flash => {
                Effect::Reactive
            }
            _ => Effect::Wave,
        };
    }
    if let Some(rgb) = state.programs.as_ref().and_then(|p| p.settings().color) {
        state.palette.set_rgb(rgb);
    }
    state.replay();
}

pub(super) fn island(ui: &mut Ui, state: &mut State) {
    super::program_cards::show(ui, state);
}

pub(super) fn controls(ui: &mut Ui, state: &mut State) {
    let Some(programs) = &mut state.programs else {
        return;
    };
    let p = programs.preset_mut();
    if let Some(max) = p.brightness_max {
        crate::ui::widgets::lighting_slider::lighting_slider(
            ui,
            s::STUDIO_BRIGHTNESS,
            &mut p.saved.brightness,
            0..=max,
            "",
        );
    }
    if let Some(max) = p.speed_max {
        crate::ui::widgets::lighting_slider::lighting_slider(
            ui,
            s::LIGHTING_SPEED,
            &mut p.saved.speed,
            0..=max,
            "",
        )
        .on_hover_text(s::LIGHTING_SPEED_HINT);
    }
}

pub(super) fn brightness(state: &State) -> f32 {
    state.programs.as_ref().map_or(1.0, |p| {
        if p.preset().effect == LightingEffect::Off {
            0.0
        } else {
            p.preset().brightness_max.map_or(1.0, |max| {
                f32::from(p.settings().brightness) / f32::from(max.max(1))
            })
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_and_drafts_never_move_the_device_checkmark_and_failure_keeps_it() {
        let source = LightingEditor {
            current: 1,
            presets: (1..=2)
                .map(|preset| LightingPreset {
                    device_color: None,
                    effect: LightingEffect::Breathing,
                    saved: LightingSettings {
                        preset,
                        brightness: 3,
                        speed: 2,
                        color: None,
                    },
                    brightness_max: Some(4),
                    speed_max: Some(4),
                    custom_color: true,
                })
                .collect(),
        };
        let mut state = State::default();
        sync(&mut state, Some(&source));
        let p = state.programs.as_mut().unwrap();
        assert!(
            !p.is_applied(source.current),
            "offline catalog has no device state"
        );
        p.observe_device(Some(source.current), false);
        p.selected = 1;
        p.preset_mut().saved.brightness = 1;
        let draft = p.settings();
        assert!(p.is_applied(source.current));
        p.observe_device(Some(draft.preset), true);
        assert!(p.is_applied(source.current));
        p.observe_device(Some(draft.preset), false);
        assert!(p.is_applied(draft.preset));
        assert_eq!(p.settings(), draft);
        p.observe_device(None, false);
        assert!(!p.is_applied(draft.preset));
    }

    #[test]
    fn drafts_survive_frames_and_mode_switches_until_a_confirmed_device_update() {
        let mut source = LightingEditor {
            current: 5,
            presets: vec![LightingPreset {
                device_color: None,
                effect: LightingEffect::Solid,
                saved: LightingSettings {
                    preset: 5,
                    brightness: 3,
                    speed: 0,
                    color: None,
                },
                brightness_max: Some(4),
                speed_max: None,
                custom_color: true,
            }],
        };
        let mut state = State::default();
        sync(&mut state, Some(&source));
        state
            .programs
            .as_mut()
            .unwrap()
            .preset_mut()
            .saved
            .brightness = 0;
        sync(&mut state, Some(&source));
        assert_eq!(brightness(&state), 0.0);
        state.enter(super::super::Mode::Encoder);
        state.enter(super::super::Mode::Lighting);
        assert_eq!(brightness(&state), 0.0);
        source.presets[0].saved.brightness = 2;
        sync(&mut state, Some(&source));
        assert_eq!(brightness(&state), 0.0);
    }
}
