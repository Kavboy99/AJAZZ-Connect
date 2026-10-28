//! Native program drafts stay local until the explicit device action is pressed.
use super::State;
use crate::core::{LightingEditor, LightingEffect, LightingPreset, LightingSettings};

use crate::ui::{strings as s, viewer::Effect};
use eframe::egui::Ui;

pub(super) struct Programs {
    source: LightingEditor,
    pub presets: Vec<LightingPreset>,
    pub selected: usize,
}

impl Programs {
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
        state.programs = Some(Programs {
            source: source.clone(),
            presets: source.presets.clone(),
            selected,
        });
        update_preview(state);
    }
}

pub(super) fn update_preview(state: &mut State) {
    if let Some(p) = &state.programs {
        state.effect = match p.preset().effect {
            LightingEffect::Solid | LightingEffect::Off => Effect::Solid,
            LightingEffect::Reactive | LightingEffect::Ripples | LightingEffect::Fade => {
                Effect::Reactive
            }
            _ => Effect::Wave,
        };
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
        ui.add(
            eframe::egui::Slider::new(&mut p.saved.brightness, 0..=max).text(s::STUDIO_BRIGHTNESS),
        );
    }
    let speed_changed = p.speed_max.is_some_and(|max| {
        ui.add(eframe::egui::Slider::new(&mut p.saved.speed, 0..=max).text(s::LIGHTING_SPEED))
            .changed()
    });
    if speed_changed {
        state.replay();
    }
    ui.label(if state.preview_requires_usb {
        s::LIGHTING_PREVIEW_ONLY
    } else {
        s::LIGHTING_DRAFT_HINT
    });
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
    fn drafts_survive_frames_and_mode_switches_until_a_confirmed_device_update() {
        let mut source = LightingEditor {
            current: 5,
            presets: vec![LightingPreset {
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
        assert_eq!(brightness(&state), 0.5);
    }
}
