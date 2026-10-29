use super::*;
use crate::core::LightingSettings;

#[test]
fn vendor_and_owner_confirmed_brightness_and_speed_bytes_are_independent() {
    for preset_id in [2, 7] {
        let mut raw = config();
        let at = 0x38 + 2 * usize::from(preset_id);
        raw[0x0a] = preset_id;
        raw[at] = 3;
        raw[at + 1] = 0x17;
        let original = Config::parse(&raw).unwrap();
        assert_eq!(original.settings(preset_id), (1, 3, 7));
        let colors = ColorTable::parse(&colors()).unwrap();
        let preset = super::super::lighting::editor(&original, &colors)
            .presets
            .into_iter()
            .find(|p| p.saved.preset == preset_id)
            .unwrap();
        assert_eq!((preset.saved.brightness, preset.saved.speed), (3, 1));
        for (brightness, speed, changed_at, expected) in [(4, 1, at, 4), (3, 2, at + 1, 0x27)] {
            let changed = original.with_program(
                LightingSettings {
                    preset: preset_id,
                    brightness,
                    speed,
                    color: None,
                },
                true,
                true,
            );
            assert_eq!(changed.bytes()[changed_at], expected);
            for (index, value) in raw.iter().enumerate().filter(|(i, _)| *i != changed_at) {
                assert_eq!(changed.bytes()[index], *value, "unrelated byte {index}");
            }
        }
    }
}

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
fn explicit_native_commit_confirms_each_block_and_preserves_unrelated_fields() {
    let (fake, state) = keyboard(true);
    let bus: Arc<dyn Bus> = fake.clone();
    let mut device = ByTechDriver.discover(&bus).devices.remove(0);
    let original = (
        Config::parse(&config()).unwrap(),
        ColorTable::parse(&colors()).unwrap(),
    );
    let settings = LightingSettings {
        preset: 7,
        brightness: 2,
        speed: 1,
        color: Some(Rgb::new(10, 20, 30)),
    };
    let target = super::super::lighting::target(&original, settings).unwrap();
    let before = fake.log().len();
    let lighting = device.lighting().unwrap();
    assert_eq!(
        lighting.apply_settings(settings),
        Ok(crate::core::Outcome::Confirmed)
    );
    assert_eq!(
        opcodes(&fake.log()[before..]),
        [0x82, 0x84, 0x8a, 0x0a, 0x8a, 0x84, 0x04, 0x84, 0x8a]
    );
    assert_eq!(lighting.editor().unwrap().current, 7);
    assert_eq!(fake.backups().len(), 1);
    let k = state.lock().unwrap();
    assert_eq!(k.config, target.0.bytes());
    assert_eq!(k.colors, target.1.bytes());
}

#[test]
fn failed_palette_verification_never_selects_the_new_mode_or_retries() {
    let (fake, _) = keyboard(false);
    let bus: Arc<dyn Bus> = fake.clone();
    let mut device = ByTechDriver.discover(&bus).devices.remove(0);
    let lighting = device.lighting().unwrap();
    assert_eq!(
        lighting.apply_settings(LightingSettings {
            preset: 7,
            brightness: 2,
            speed: 1,
            color: Some(Rgb::new(10, 20, 30))
        }),
        Err(DriverError::ReadBackMismatch)
    );
    assert_eq!(lighting.editor().unwrap().current, 3);
    let writes: Vec<_> = opcodes(&fake.log())
        .into_iter()
        .filter(|op| op & 0x80 == 0)
        .collect();
    assert_eq!(writes, [0x0a]);
}

#[test]
fn failed_backup_or_changed_identity_prevents_any_write() {
    for foreign in [false, true] {
        let (fake, state) = keyboard(true);
        let bus: Arc<dyn Bus> = fake.clone();
        let mut device = ByTechDriver.discover(&bus).devices.remove(0);
        if foreign {
            state.lock().unwrap().password = PASSWORDS[0];
        } else {
            fake.fail_backups();
        }
        assert!(
            device
                .lighting()
                .unwrap()
                .apply_settings(LightingSettings {
                    preset: 7,
                    brightness: 2,
                    speed: 1,
                    color: None
                })
                .is_err()
        );
        assert!(opcodes(&fake.log()).into_iter().all(|op| op & 0x80 != 0));
    }
}

#[test]
fn config_only_commits_do_not_write_the_palette_and_failed_verification_is_not_retried() {
    for stores in [false, true] {
        let (fake, _) = keyboard(stores);
        let bus: Arc<dyn Bus> = fake.clone();
        let mut device = ByTechDriver.discover(&bus).devices.remove(0);
        let lighting = device.lighting().unwrap();
        let result = lighting.apply_settings(LightingSettings {
            preset: 7,
            brightness: 2,
            speed: 1,
            color: None,
        });
        assert_eq!(
            result,
            if stores {
                Ok(crate::core::Outcome::Confirmed)
            } else {
                Err(DriverError::ReadBackMismatch)
            }
        );
        assert_eq!(
            lighting.editor().unwrap().current,
            if stores { 7 } else { 3 }
        );
        let writes: Vec<_> = opcodes(&fake.log())
            .into_iter()
            .filter(|op| op & 0x80 == 0)
            .collect();
        assert_eq!(writes, [4]);
    }
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
    assert_eq!(target.0.bytes()[0x46], 2);
    assert_eq!(target.0.bytes()[0x47], 0x10);
}

#[test]
fn cross_block_corruption_is_detected_without_retry_or_advancing_the_cached_mode() {
    for corrupt_after in [0x0a, 0x04] {
        let (_, state) = keyboard(true);
        let mut inner = responder(state.clone());
        let fake = FakeBus::new(
            vec![endpoint()],
            Box::new(move |packet| {
                let reply = inner(packet);
                if matches!(packet, Packet::Feature(p) if p[1] == corrupt_after) {
                    let mut k = state.lock().unwrap();
                    if corrupt_after == 0x0a {
                        k.config[0] ^= 1;
                    } else {
                        k.colors[0] ^= 1;
                    }
                }
                reply
            }),
        );
        let bus: Arc<dyn Bus> = fake.clone();
        let mut device = ByTechDriver.discover(&bus).devices.remove(0);
        let lighting = device.lighting().unwrap();
        assert!(
            lighting
                .apply_settings(LightingSettings {
                    preset: 7,
                    brightness: 2,
                    speed: 1,
                    color: Some(Rgb::new(10, 20, 30))
                })
                .is_err()
        );
        assert_eq!(lighting.editor().unwrap().current, 3);
        let writes: Vec<_> = opcodes(&fake.log())
            .into_iter()
            .filter(|op| op & 0x80 == 0)
            .collect();
        assert_eq!(
            writes,
            if corrupt_after == 0x0a {
                vec![0x0a]
            } else {
                vec![0x0a, 4]
            }
        );
    }
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
