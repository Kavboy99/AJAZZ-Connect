use super::codec::{self, GET_INFO, GET_LED_EFFECT, SET_LED_EFFECT};
use super::info::KeyboardInfo;
use super::*;
use crate::core::{Bus, Driver, DriverError, Outcome, ReportSizes, Rgb};
use crate::drivers::fake_bus::{FakeBus, Packet, Responder};
use std::sync::Arc;

fn effect() -> [u8; 16] {
    let mut e = [0; 16];
    e[0] = 7;
    e[5..8].copy_from_slice(&[10, 11, 12]);
    e[9] = 3;
    e[10] = 4;
    e[12] = 0x34;
    e[13] = 0x56;
    e[14] = 0xaa;
    e[15] = 0x55;
    e
}

fn info_bytes() -> [u8; 48] {
    let mut b = [0; 48];
    b[4..10].copy_from_slice(&[0x45, 0x0c, 0xa0, 0x80, 0x36, 1]);
    b[12..16].copy_from_slice(&[0x66, 1, 4, 0x10]);
    b
}

fn reply(command: u8, payload: &[u8]) -> Vec<u8> {
    let mut r = vec![0; 65];
    r[1..4].copy_from_slice(&[0x55, command, payload.len() as u8]);
    r[9..9 + payload.len()].copy_from_slice(payload);
    r
}

/// A keyboard that answers like the researched firmware and keeps the written effect.
fn keyboard() -> Responder {
    let mut stored = effect().to_vec();
    Box::new(move |packet| {
        let Packet::Output(p) = packet else {
            return Vec::new();
        };
        match p[2] {
            GET_INFO => vec![reply(GET_INFO, &info_bytes())],
            GET_LED_EFFECT => vec![reply(GET_LED_EFFECT, &stored)],
            SET_LED_EFFECT => {
                stored = p[9..25].to_vec();
                vec![reply(SET_LED_EFFECT, &stored)]
            }
            _ => Vec::new(),
        }
    })
}

fn endpoint() -> Endpoint {
    Endpoint::hid(0x0c45, 0x80a0, 0xff68, 0x61)
        .with_product("AK820 MAX")
        .with_reports(Some(ReportSizes {
            input: 65,
            output: 65,
            feature: 0,
        }))
}

#[test]
fn allows_only_stock_lighting_commands_and_preserves_unknown_bytes() {
    let p = codec::packet(GET_INFO, None).unwrap();
    assert_eq!(&p[..9], &[0, 0xaa, 0x10, 48, 0, 0, 0, 1, 0]);
    for cmd in [0x0f, 0x24, 0x4f, 0x80, 0x81, 0x86] {
        assert!(codec::packet(cmd, None).is_err());
    }
    let original = effect();
    let changed = codec::solid_payload(&original, [10, 20, 30]).unwrap();
    assert_eq!(&changed[..5], &[1, 10, 20, 30, 255]);
    for i in [5, 6, 7, 9, 10, 11, 12, 13, 14, 15] {
        assert_eq!(changed[i], original[i]);
    }
    let mut ack = [0; 65];
    ack[1..6].copy_from_slice(&[0x55, 0x23, 16, 0, 0]);
    ack[9..25].copy_from_slice(&changed);
    assert_eq!(codec::response(&ack, 0x23, 16).unwrap(), changed);
    assert!(codec::response(&ack, 0x13, 16).is_err());
    ack[4] = 1;
    assert!(codec::response(&ack, 0x23, 16).is_err());
}

#[test]
fn identity_tracks_firmware_and_profile_but_not_battery() {
    let mut info = [0; 48];
    info[4..10].copy_from_slice(&[0x45, 0xc, 9, 0x80, 7, 1]);
    let base = codec::identity(&info).unwrap();
    info[17] = 79;
    assert_eq!(codec::identity(&info).unwrap(), base);
    info[19] = 1;
    assert_ne!(codec::identity(&info).unwrap(), base);
    info[19] = 0;
    info[8] = 8;
    assert_ne!(codec::identity(&info).unwrap(), base);
}

#[test]
fn parses_known_info_and_rejects_malformed_version() {
    let mut b = info_bytes();
    let p = KeyboardInfo::parse(&b).unwrap();
    assert_eq!(
        (p.vid, p.pid, p.manufacturer, p.product),
        (0xc45, 0x80a0, 0x166, 0x1004)
    );
    assert_eq!(p.version, "1.36");
    b[8] = 0xfa;
    assert!(KeyboardInfo::parse(&b).is_err());
    assert!(KeyboardInfo::parse(&b[..47]).is_err());
}

#[test]
fn admission_needs_shape_profile_and_never_takes_the_bootloader() {
    assert!(admits(&endpoint()));
    let mut wrong_size = endpoint();
    wrong_size.reports = Some(ReportSizes {
        input: 65,
        output: 4097,
        feature: 0,
    });
    assert!(!admits(&wrong_size));
    assert!(!admits(&endpoint().with_product("Gaming Keyboard")));
    let mut bootloader = endpoint();
    bootloader.pid = 0x7210;
    assert!(!admits(&bootloader));
}

#[test]
fn discovery_only_reads_and_backup_precedes_the_first_write() {
    let fake = FakeBus::new(vec![endpoint()], keyboard());
    let bus: Arc<dyn Bus> = fake.clone();
    let mut found = Aa55Driver.discover(&bus);
    assert_eq!(found.devices.len(), 1, "{:?}", found.notes);
    let commands = |log: &[Packet]| -> Vec<u8> {
        log.iter()
            .filter_map(|p| match p {
                Packet::Output(p) => Some(p[2]),
                Packet::Feature(_) => None,
            })
            .collect()
    };
    assert_eq!(commands(&fake.log()), vec![GET_INFO, GET_LED_EFFECT]);

    let lighting = found.devices[0].lighting().unwrap();
    assert_eq!(lighting.restore(), Err(DriverError::NoRollback));
    assert_eq!(
        lighting.apply_solid(Rgb::new(10, 20, 30)),
        Ok(Outcome::Confirmed)
    );
    let log = commands(&fake.log());
    let first_set = log.iter().position(|c| *c == SET_LED_EFFECT).unwrap();
    assert!(fake.packets_before_backup().is_some_and(|n| n <= first_set));
    assert_eq!(fake.backups().len(), 1);
    assert_eq!(lighting.restore(), Ok(Outcome::Restored));
}
