use super::*;
use crate::core::{LightingEffect, LightingSettings};

#[test]
fn offline_and_live_catalogs_share_controls_without_hardware_discovery() {
    let preview = ByTechDriver.lighting_preview().unwrap();
    assert!(preview.requires_usb);
    assert_eq!(preview.editor.presets.len(), 19);
    let (fake, _) = keyboard(true);
    assert!(fake.log().is_empty());
    let bus: Arc<dyn Bus> = fake.clone();
    let mut device = ByTechDriver.discover(&bus).devices.remove(0);
    let actual = device.lighting().unwrap().editor().unwrap();
    for (offline, live) in preview.editor.presets.iter().zip(actual.presets) {
        assert_eq!(offline.effect, live.effect);
        assert_eq!(offline.saved.preset, live.saved.preset);
        assert_eq!(offline.brightness_max, live.brightness_max);
        assert_eq!(offline.speed_max, live.speed_max);
        assert_eq!(offline.custom_color, live.custom_color);
    }
}

#[test]
fn native_programs_preserve_other_modes_and_palette_and_allow_zero_brightness() {
    let (fake, state) = keyboard(true);
    let bus: Arc<dyn Bus> = fake.clone();
    let mut device = ByTechDriver.discover(&bus).devices.remove(0);
    let lighting = device.lighting().unwrap();
    let editor = lighting.editor().unwrap();
    assert_eq!(editor.current, 3);
    assert_eq!(editor.presets.len(), 19);
    assert_eq!(editor.presets.last().unwrap().effect, LightingEffect::Off);
    let before = fake.log().len();
    let s = LightingSettings {
        preset: 7,
        brightness: 0,
        speed: 4,
        color: None,
    };
    assert_eq!(lighting.apply_settings(s).unwrap(), Outcome::Confirmed);
    assert_eq!(
        opcodes(&fake.log()[before..]),
        [0x82, 0x84, 0x8a, 0x04, 0x84, 0x8a]
    );
    let first_write = fake
        .log()
        .iter()
        .position(|p| matches!(p, Packet::Feature(p) if p[1] == 4))
        .unwrap();
    assert!(fake.packets_before_backup().unwrap() <= first_write);
    let mut expected = config();
    expected[0x0a] = 7;
    expected[0x46] = 4;
    assert_eq!(state.lock().unwrap().config, expected);
    assert_eq!(state.lock().unwrap().colors, colors());
    assert_eq!(lighting.editor().unwrap().current, 7);
    assert_eq!(lighting.restore().unwrap(), Outcome::Restored);
    assert_eq!(lighting.editor().unwrap().current, 3);
    assert_eq!(state.lock().unwrap().config, config());
}

#[test]
fn every_native_program_round_trips_with_its_saved_values() {
    let (fake, state) = keyboard(true);
    let bus: Arc<dyn Bus> = fake.clone();
    let mut device = ByTechDriver.discover(&bus).devices.remove(0);
    let lighting = device.lighting().unwrap();
    for preset in lighting.editor().unwrap().presets {
        assert_eq!(
            lighting.apply_settings(preset.saved).unwrap(),
            Outcome::Confirmed
        );
        assert_eq!(state.lock().unwrap().config[0x0a], preset.saved.preset);
    }
    assert_eq!(fake.backups().len(), 1);
}

#[test]
fn native_color_changes_only_slot_zero_of_the_selected_effect() {
    let current = (
        Config::parse(&config()).unwrap(),
        ColorTable::parse(&colors()).unwrap(),
    );
    let target = super::super::lighting::target(
        &current,
        LightingSettings {
            preset: 7,
            brightness: 2,
            speed: 1,
            color: Some(Rgb::new(10, 20, 30)),
        },
    )
    .unwrap();
    let at = 21 * 7;
    assert_eq!(&target.1.bytes()[at..at + 3], &[10, 20, 30]);
    assert_eq!(&target.1.bytes()[..at], &current.1.bytes()[..at]);
    assert_eq!(&target.1.bytes()[at + 3..], &current.1.bytes()[at + 3..]);
    assert_eq!(target.0.bytes()[0x47], 0x20);
}

#[test]
fn invalid_native_settings_send_no_writes() {
    let (fake, _) = keyboard(true);
    let bus: Arc<dyn Bus> = fake.clone();
    let mut device = ByTechDriver.discover(&bus).devices.remove(0);
    let lighting = device.lighting().unwrap();
    for (preset, brightness, speed, color) in [
        (19, 1, 1, None),
        (22, 1, 1, None),
        (7, 5, 1, None),
        (7, 1, 5, None),
        (3, 1, 1, Some(Rgb::new(1, 2, 3))),
    ] {
        assert!(
            lighting
                .apply_settings(LightingSettings {
                    preset,
                    brightness,
                    speed,
                    color
                })
                .is_err()
        );
    }
    assert!(opcodes(&fake.log()).iter().all(|op| op & 0x80 != 0));
    assert!(fake.backups().is_empty());
}

#[test]
fn native_readback_failure_is_not_reported_as_success_or_retried() {
    let (fake, _) = keyboard(false);
    let bus: Arc<dyn Bus> = fake.clone();
    let mut device = ByTechDriver.discover(&bus).devices.remove(0);
    let lighting = device.lighting().unwrap();
    assert_eq!(
        lighting
            .apply_settings(LightingSettings {
                preset: 7,
                brightness: 3,
                speed: 2,
                color: None,
            })
            .unwrap_err(),
        DriverError::ReadBackMismatch
    );
    assert_eq!(lighting.editor().unwrap().current, 3);
    assert_eq!(
        opcodes(&fake.log()).iter().filter(|op| **op == 4).count(),
        1
    );
}
