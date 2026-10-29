//! `KB.ini` LedOpt1..19, LightHW and SpeedHW from the Wired/Thr vendor profiles.
use super::state::{ColorTable, Config};
use crate::core::{LightingEditor, LightingEffect as E, LightingPreset, LightingSettings};

const EFFECTS: [E; 19] = [
    E::Solid,
    E::Breathing,
    E::Rainbow,
    E::Flash,
    E::Rain,
    E::RainbowWheel,
    E::Ripples,
    E::Stars,
    E::Fade,
    E::Snake,
    E::Neon,
    E::Reactive,
    E::Sine,
    E::Scan,
    E::Windmill,
    E::Waterfall,
    E::Blossom,
    E::Storm,
    E::Off,
];

pub(super) fn editor(config: &Config, colors: &ColorTable) -> LightingEditor {
    let mut editor = catalog(config.effect(), |id| {
        let (speed, brightness, _) = config.settings(id);
        (speed, brightness)
    });
    for preset in &mut editor.presets {
        let (_, _, slot) = config.settings(preset.saved.preset);
        if slot < 7 {
            let at = usize::from(preset.saved.preset) * 21 + usize::from(slot) * 3;
            let rgb = colors.bytes();
            preset.device_color = Some(crate::core::Rgb::new(rgb[at], rgb[at + 1], rgb[at + 2]));
        }
    }
    editor
}

pub(super) fn preview() -> crate::core::LightingPreview {
    crate::core::LightingPreview {
        editor: catalog(1, |_| (2, 4)),
        requires_usb: true,
    }
}

fn catalog(current: u8, settings: impl Fn(u8) -> (u8, u8)) -> LightingEditor {
    LightingEditor {
        current,
        presets: EFFECTS
            .iter()
            .enumerate()
            .map(|(index, &effect)| {
                let id = if effect == E::Off {
                    21
                } else {
                    index as u8 + 1
                };
                let (speed, brightness) = settings(id);
                LightingPreset {
                    device_color: None,
                    effect,
                    saved: LightingSettings {
                        preset: id,
                        brightness,
                        speed,
                        color: None,
                    },
                    brightness_max: (effect != E::Off).then_some(4),
                    speed_max: (!matches!(effect, E::Solid | E::Off)).then_some(4),
                    custom_color: !matches!(id, 3 | 15 | 16 | 17 | 21),
                }
            })
            .collect(),
    }
}

pub(super) fn target(
    current: &(Config, ColorTable),
    settings: LightingSettings,
) -> Result<(Config, ColorTable), crate::core::DriverError> {
    let preset = editor(&current.0, &current.1)
        .presets
        .into_iter()
        .find(|p| p.saved.preset == settings.preset)
        .ok_or_else(|| crate::core::DriverError::rejected("unknown lighting preset"))?;
    if preset
        .brightness_max
        .is_some_and(|max| settings.brightness > max)
        || preset.speed_max.is_some_and(|max| settings.speed > max)
        || (settings.color.is_some() && !preset.custom_color)
    {
        return Err(crate::core::DriverError::rejected(
            "unsupported lighting setting",
        ));
    }
    let config = current.0.with_program(
        settings,
        preset.brightness_max.is_some(),
        preset.speed_max.is_some(),
    );
    let colors = settings.color.map_or_else(
        || current.1.clone(),
        |color| {
            current
                .1
                .with_effect_color(settings.preset, color.to_array())
        },
    );
    Ok((config, colors))
}
