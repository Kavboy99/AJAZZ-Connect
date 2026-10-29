use super::State;
use crate::app::Settings;

pub(super) fn restore(state: &mut State, settings: &Settings, key: &str) {
    let Some(programs) = &mut state.programs else {
        return;
    };
    if let Some(saved) = settings.lighting.get(key) {
        for preset in &mut programs.presets {
            if let Some(value) = saved.iter().find(|v| v.preset == preset.saved.preset) {
                preset.saved.brightness = value.brightness.min(preset.brightness_max.unwrap_or(0));
                preset.saved.speed = value.speed.min(preset.speed_max.unwrap_or(0));
                if preset.custom_color {
                    preset.saved.color = value.color;
                }
            }
        }
    }
    if let Some(selected) = settings.lighting_selected.get(key)
        && let Some(index) = programs
            .presets
            .iter()
            .position(|p| p.saved.preset == *selected)
    {
        programs.selected = index;
    }
    super::programs::update_preview(state);
}

pub(super) fn save(state: &State, settings: &mut Settings, key: &str) -> std::io::Result<()> {
    let Some(programs) = &state.programs else {
        return Ok(());
    };
    let values: Vec<_> = programs.presets.iter().map(|p| p.saved).collect();
    let selected = programs.settings().preset;
    if settings.lighting.get(key) != Some(&values)
        || settings.lighting_selected.get(key) != Some(&selected)
    {
        settings.lighting.insert(key.to_owned(), values);
        settings.lighting_selected.insert(key.to_owned(), selected);
        settings.save()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{LightingEditor, LightingEffect, LightingPreset, LightingSettings, Rgb};
    #[test]
    fn app_preferences_override_device_state_and_validate_limits() {
        let mut state = State::default();
        let device = LightingSettings {
            preset: 2,
            brightness: 1,
            speed: 1,
            color: Some(Rgb::new(255, 0, 0)),
        };
        super::super::programs::sync(
            &mut state,
            Some(&LightingEditor {
                current: 2,
                presets: vec![LightingPreset {
                    effect: LightingEffect::Solid,
                    saved: device,
                    device_color: device.color,
                    brightness_max: Some(4),
                    speed_max: Some(5),
                    custom_color: true,
                }],
            }),
        );
        let mut preferences = Settings::default();
        preferences.lighting.insert(
            "model".into(),
            vec![LightingSettings {
                brightness: 255,
                speed: 255,
                color: Some(Rgb::new(255, 255, 255)),
                ..device
            }],
        );
        restore(&mut state, &preferences, "model");
        let selected = state.programs.as_ref().unwrap().settings();
        assert_eq!(selected.brightness, 4);
        assert_eq!(selected.speed, 5);
        assert_eq!(state.palette.rgb(), Rgb::new(255, 255, 255));
        super::super::programs::sync(
            &mut state,
            Some(&LightingEditor {
                current: 2,
                presets: vec![],
            }),
        );
        assert_eq!(state.programs.as_ref().unwrap().settings(), selected);
    }
}
