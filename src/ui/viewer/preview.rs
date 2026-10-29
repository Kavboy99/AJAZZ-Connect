//! Visual bindings are asset data, independent of hardware capabilities and protocol drivers.
use super::glb::Scene;
use eframe::egui::{Color32, Key, Modifiers, ecolor};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Effect {
    #[default]
    Solid,
    Wave,
    Reactive,
}
impl Effect {
    pub fn color_count(self) -> usize {
        if self == Self::Wave { 3 } else { 1 }
    }
}

pub struct Preview {
    pub hits: HashMap<Key, f32>,
    pub native: Option<crate::core::LightingEffect>,
    pub brightness: f32,
    pub effect: Option<Effect>,
    pub colors: Vec<Color32>,
    pub phase: f32,
    pub keys: HashMap<Key, f32>,
    pub modifiers: Modifiers,
}

#[derive(Clone, Copy, Default)]
pub(super) struct PartEffect {
    pub emission: [f32; 3],
    pub press: f32,
    pub socket: bool,
}

impl PartEffect {
    pub fn key(self) -> [u8; 4] {
        let mut bytes = [0; 4];
        for (out, value) in bytes.iter_mut().zip(self.emission) {
            *out = (value.clamp(0.0, 8.0) * 31.0).round() as u8;
        }
        bytes[3] = (self.press * 255.0).round() as u8;
        bytes
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Stages {
    stage: Vec<Binding>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    visual: String,
    stage_aspect: f32,
    key_prefix: String,
    socket_prefix: String,
    encoder_parts: Vec<String>,
    encoder_focus: [f32; 3],
    keys: Vec<String>,
}

pub fn binding(visual: Option<&str>) -> Option<&'static Binding> {
    let visual = visual.map(|v| {
        if matches!(v, "ak820-purple" | "ak820-white") {
            "ak820-max"
        } else {
            v
        }
    });
    static BINDINGS: OnceLock<Option<Stages>> = OnceLock::new();
    BINDINGS
        .get_or_init(|| toml::from_str(include_str!("../../../assets/models/stages.toml")).ok())
        .as_ref()?
        .stage
        .iter()
        .find(|b| Some(b.visual.as_str()) == visual)
}

impl Binding {
    pub(super) fn key_label(&self, index: usize) -> Option<&str> {
        self.keys.get(index).map(String::as_str)
    }
    pub(super) fn key_index(&self, name: &str) -> Option<usize> {
        name.strip_prefix(&self.key_prefix)?.parse().ok()
    }
    pub(super) fn socket_index(&self, name: &str) -> Option<usize> {
        name.strip_prefix(&self.socket_prefix)?.parse().ok()
    }
    pub fn stage_aspect(&self) -> f32 {
        self.stage_aspect
    }
    pub fn has_lighting(&self) -> bool {
        !self.keys.is_empty()
    }
    pub fn has_encoder(&self) -> bool {
        !self.encoder_parts.is_empty()
    }
    pub fn encoder_focus(&self) -> [f32; 3] {
        self.encoder_focus
    }

    pub(super) fn effects(&self, scene: &Scene, preview: &Preview) -> Vec<PartEffect> {
        scene
            .parts
            .iter()
            .map(|name| {
                let Some(name) = name else {
                    return PartEffect::default();
                };
                let key = name.strip_prefix(&self.key_prefix);
                let socket = name.strip_prefix(&self.socket_prefix);
                let Some(index) = key.or(socket).and_then(|s| s.parse::<usize>().ok()) else {
                    return PartEffect::default();
                };
                let Some(name) = self.keys.get(index) else {
                    return PartEffect::default();
                };
                let press = match name.as_str() {
                    "Shift" => f32::from(preview.modifiers.shift),
                    "Ctrl" => f32::from(preview.modifiers.ctrl),
                    "Alt" => f32::from(preview.modifiers.alt),
                    "Super" => f32::from(preview.modifiers.mac_cmd),
                    name => Key::from_name(name)
                        .and_then(|k| preview.keys.get(&k).copied())
                        .unwrap_or(0.0),
                };
                let color = preview.colors.first().copied().unwrap_or(Color32::WHITE);
                let mut persistence = 1.0;
                let rgb = if let Some(effect) = preview.native {
                    if effect.key_triggered() {
                        let mut strength: f32 = 0.0;
                        for (source, name) in self.keys.iter().enumerate() {
                            let Some(age) = Key::from_name(name)
                                .and_then(|k| preview.hits.get(&k))
                                .copied()
                            else {
                                continue;
                            };
                            let light = super::reactive::strength(effect, index, source, age);
                            strength = strength.max(light);
                        }
                        persistence = strength;
                        color
                    } else {
                        super::native_effect::sample(
                            effect,
                            (index % 14) as f32 / 13.0,
                            (index / 14) as f32 / 5.0,
                            preview.phase,
                            press,
                            color,
                        )
                    }
                } else if preview.effect == Some(Effect::Wave) && preview.colors.len() > 1 {
                    let t = (index as f32 / 14.0 - preview.phase).rem_euclid(1.0)
                        * preview.colors.len() as f32;
                    let a = preview.colors[t.floor() as usize % preview.colors.len()];
                    let b = preview.colors[(t.floor() as usize + 1) % preview.colors.len()];
                    a.lerp_to_gamma(b, t.fract())
                } else {
                    color
                };
                // Persistence is radiance, not an sRGB color edit: gamma scaling crushed the wake.
                let strength = preview.brightness
                    * persistence
                    * if preview.native.is_some() {
                        4.0
                    } else {
                        match preview.effect {
                            Some(Effect::Solid | Effect::Wave) => 4.0 + press * 0.8,
                            Some(Effect::Reactive) | None => press * 4.8,
                        }
                    };
                PartEffect {
                    emission: [rgb.r(), rgb.g(), rgb.b()]
                        .map(|v| ecolor::linear_f32_from_gamma_u8(v) * strength),
                    socket: socket.is_some(),
                    press: if key.is_some() { press } else { 0.0 },
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bindings_name_real_parts_and_valid_keys() {
        let stages: Stages =
            toml::from_str(include_str!("../../../assets/models/stages.toml")).unwrap();
        for b in stages.stage {
            let bytes = super::super::catalog::model_bytes(&b.visual).unwrap();
            let scene = super::super::glb::parse(bytes).unwrap();
            for (index, key) in b.keys.iter().enumerate() {
                assert!(
                    scene
                        .parts
                        .iter()
                        .flatten()
                        .any(|n| n == &format!("{}{index}", b.key_prefix))
                );
                assert!(
                    Key::from_name(key).is_some()
                        || ["Shift", "Ctrl", "Alt", "Super", "Fn", "CapsLock"]
                            .contains(&key.as_str()),
                    "{key}"
                );
            }
            assert!(
                b.encoder_parts
                    .iter()
                    .all(|name| scene.parts.iter().flatten().any(|p| p == name))
            );
        }
    }
}
