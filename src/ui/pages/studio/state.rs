//! Ephemeral editor state. Input is scoped to the focused window and is never recorded.
use crate::ui::motion::{self, Channel, tokens};
use crate::ui::viewer::{Effect, Preview};
use crate::ui::widgets::lighting_palette::Palette;
use eframe::egui::{Id, Key, Ui};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Mode {
    #[default]
    Overview,
    Lighting,
    Encoder,
}

pub(crate) struct State {
    pub mode: Mode,
    pub inspector: bool,
    pub effect: Effect,
    pub palette: Palette,
    pub encoder: usize,
    pub step: u8,
    pub press: bool,
    pub hardware: super::super::lighting::State,
    pub(super) programs: Option<super::programs::Programs>,
    pub preview_requires_usb: bool,
    panel_is_inspector: bool,
    demo_left: f32,
    keys: HashMap<Key, f32>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            mode: Mode::Overview,
            inspector: false,
            effect: Effect::Solid,
            palette: Palette::default(),
            encoder: 0,
            step: 5,
            press: false,
            hardware: super::super::lighting::State::default(),
            programs: None,
            preview_requires_usb: false,
            panel_is_inspector: false,
            demo_left: 0.0,
            keys: HashMap::new(),
        }
    }
}

impl State {
    pub fn choice_label(&self) -> &'static str {
        use crate::ui::strings as s;
        if self.mode == Mode::Encoder {
            [s::STUDIO_VOLUME, s::STUDIO_SCROLL, s::STUDIO_BRIGHTNESS][self.encoder]
        } else {
            if let Some(p) = &self.programs {
                return s::lighting_effect(p.preset().effect);
            }
            match self.effect {
                Effect::Solid => s::STUDIO_SOLID,
                Effect::Wave => s::STUDIO_WAVE,
                Effect::Reactive => s::STUDIO_REACTIVE,
            }
        }
    }
    pub fn enter(&mut self, mode: Mode) {
        self.mode = mode;
        self.inspector = false;
        self.panel_is_inspector = false;
        self.replay();
    }

    pub fn back(&mut self) -> bool {
        if self.inspector {
            self.inspector = false;
            false
        } else if self.mode != Mode::Overview {
            self.mode = Mode::Overview;
            self.panel_is_inspector = false;
            false
        } else {
            true
        }
    }

    pub fn replay(&mut self) {
        self.demo_left = if self.mode == Mode::Lighting
            && (self.effect == Effect::Wave || self.programs.is_some())
        {
            1.6
        } else {
            0.0
        };
    }

    pub fn demonstration_phase(&self) -> f32 {
        1.6 - self.demo_left
    }

    pub fn open_inspector(&mut self) {
        self.inspector = true;
        self.panel_is_inspector = true;
    }
    pub fn close_editor(&mut self) {
        self.mode = Mode::Overview;
        self.inspector = false;
        self.panel_is_inspector = false;
    }
    pub fn showing_inspector(&self) -> bool {
        self.panel_is_inspector
    }

    pub fn preview(&mut self, ui: &Ui, id: Id) -> Preview {
        let text_input = ui
            .ctx()
            .memory(|m| m.focused())
            .is_some_and(|id| eframe::egui::TextEdit::load_state(ui.ctx(), id).is_some());
        let (down, modifiers) = ui.input(|i| {
            if i.focused && ui.is_enabled() && !text_input {
                (i.keys_down.clone(), i.modifiers)
            } else {
                Default::default()
            }
        });
        for key in down.iter() {
            self.keys.insert(*key, 1.0);
        }
        for (key, strength) in &mut self.keys {
            if !down.contains(key) {
                *strength = (*strength - motion::dt(ui.ctx()) * 3.0).max(0.0);
                if *strength > 0.0 {
                    motion::keep_alive(ui.ctx());
                }
            }
        }
        self.keys.retain(|_, v| *v > 0.0);
        let phase = if motion::reduced(ui.ctx()) {
            0.0
        } else {
            1.6 - self.demo_left
        };
        if self.mode != Mode::Overview && self.demo_left > 0.0 && !motion::reduced(ui.ctx()) {
            self.demo_left = (self.demo_left - motion::dt(ui.ctx())).max(0.0);
            motion::keep_alive(ui.ctx());
        }
        let colors = self
            .palette
            .colors(
                if self
                    .programs
                    .as_ref()
                    .is_some_and(|p| p.settings().color.is_some())
                {
                    1
                } else {
                    self.effect.color_count()
                },
            )
            .into_iter()
            .enumerate()
            .map(|(index, c)| {
                motion::animate_color(ui.ctx(), id.with(("color", index)), c, tokens::FAST)
            })
            .collect();
        // The fade also supplies a static cue when spatial motion is disabled.
        let _ = motion::animate(
            ui.ctx(),
            id.with("editing"),
            f32::from(self.mode != Mode::Overview),
            tokens::PAGE,
            Channel::Fade,
        );
        Preview {
            native: if self.mode == Mode::Lighting {
                self.programs.as_ref().map(|p| p.preset().effect)
            } else {
                None
            },
            brightness: motion::animate(
                ui.ctx(),
                id.with("brightness"),
                super::programs::brightness(self),
                tokens::FAST,
                Channel::Fade,
            ),
            effect: (self.mode == Mode::Lighting).then_some(self.effect),
            colors,
            phase: phase
                * self
                    .programs
                    .as_ref()
                    .map_or(1.0, |p| 0.5 + f32::from(p.settings().speed) * 0.5),
            keys: self.keys.clone(),
            modifiers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn back_closes_inspector_then_editor_then_device() {
        let mut s = State::default();
        s.enter(Mode::Lighting);
        s.inspector = true;
        assert!(!s.back());
        assert!(!s.inspector);
        assert_eq!(s.mode, Mode::Lighting);
        assert!(!s.back());
        assert_eq!(s.mode, Mode::Overview);
        assert!(s.back());
        s.enter(Mode::Encoder);
        assert!(!s.inspector);
    }
}
