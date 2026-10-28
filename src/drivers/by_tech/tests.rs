use super::codec::{self, Command, PASSWORDS};
use super::state::{COLOR_TABLE_BYTES, CONFIG_BYTES, ColorTable, Config};
use super::*;
use crate::core::{Bus, Driver, DriverError, Outcome, ReportSizes, Rgb, Snapshot};
use crate::diagnostics::Report;
use crate::drivers::fake_bus::{FakeBus, Packet, Responder};
use std::sync::{Arc, Mutex};
mod lighting;
mod radio;

fn endpoint() -> Endpoint {
    Endpoint::hid(0x258a, 0x010c, 0xff00, 1)
        .with_product("Gaming Keyboard")
        .with_release(0x0100)
        .with_reports(Some(ReportSizes {
            input: 8,
            output: 0,
            feature: 520,
        }))
}

fn config() -> Vec<u8> {
    let mut block = vec![0; CONFIG_BYTES];
    block[0x0a] = 3;
    block[0x3a] = 2;
    block[0x3b] = 0x05;
    block[0x70] = 0x11;
    block
}

fn colors() -> Vec<u8> {
    let mut table = vec![0; COLOR_TABLE_BYTES];
    table[21..24].copy_from_slice(&[1, 2, 3]);
    table[42] = 9;
    table[0x1fa..0x1fc].copy_from_slice(&[0x5a, 0xa5]);
    table
}

/// The keyboard as the vendor software sees it: answers reads, stores writes when `stores`.
struct Keyboard {
    password: [u8; 6],
    config: Vec<u8>,
    colors: Vec<u8>,
    stores: bool,
}

/// What the keyboard answers: the echoed request header, then exactly the requested data.
fn answer(request: &[u8], data: &[u8]) -> Vec<u8> {
    let mut report = request[..8].to_vec();
    report.extend_from_slice(data);
    report
}

fn responder(state: Arc<Mutex<Keyboard>>) -> Responder {
    Box::new(move |packet| {
        let Packet::Feature(p) = packet else {
            return Vec::new();
        };
        let mut k = state.lock().unwrap();
        match p[1] {
            0x82 => vec![answer(p, &k.password)],
            0x84 => vec![answer(p, &k.config)],
            0x8a => vec![answer(p, &k.colors)],
            0x87 => vec![answer(p, &[92, 16])],
            0x04 if k.stores => {
                k.config = p[8..8 + CONFIG_BYTES].to_vec();
                Vec::new()
            }
            0x0a if k.stores => {
                k.colors = p[8..8 + COLOR_TABLE_BYTES].to_vec();
                Vec::new()
            }
            _ => Vec::new(),
        }
    })
}

fn keyboard(stores: bool) -> (Arc<FakeBus>, Arc<Mutex<Keyboard>>) {
    let state = Arc::new(Mutex::new(Keyboard {
        // The tri-mode board, as the owner's keyboard answered over USB.
        password: PASSWORDS[1],
        config: config(),
        colors: colors(),
        stores,
    }));
    (
        FakeBus::new(vec![endpoint()], responder(Arc::clone(&state))),
        state,
    )
}

fn opcodes(log: &[Packet]) -> Vec<u8> {
    log.iter()
        .filter_map(|p| match p {
            Packet::Feature(p) => Some(p[1]),
            Packet::Output(_) => None,
        })
        .collect()
}

#[test]
fn headers_match_the_vendor_requests() {
    let header = |c| codec::request(c).unwrap()[..8].to_vec();
    assert_eq!(header(Command::ReadPassword), [6, 0x82, 1, 0, 1, 0, 6, 0]);
    assert_eq!(header(Command::ReadConfig), [6, 0x84, 0, 0, 1, 0, 0x80, 0]);
    assert_eq!(header(Command::ReadColors), [6, 0x8a, 0, 0, 1, 0, 0, 2]);
    let write = codec::write(Command::WriteConfig, &config()).unwrap();
    assert_eq!(&write[..8], &[6, 0x04, 0, 0, 1, 0, 0x80, 0]);
    assert_eq!(&write[8..8 + CONFIG_BYTES], &config()[..]);
    assert!(codec::request(Command::WriteColors).is_err());
    assert!(codec::write(Command::ReadConfig, &config()).is_err());
    assert!(codec::write(Command::WriteColors, &config()).is_err());
    // The answer captured from the owner's keyboard on 2026-10-28 (Windows reports 14 bytes).
    let captured = [6, 0x82, 1, 0, 1, 0, 6, 0, 3, 0, 0, 0, 1, 0x5e];
    assert_eq!(
        codec::reply(&captured, Command::ReadPassword).unwrap(),
        &PASSWORDS[1]
    );
    let request = codec::request(Command::ReadConfig).unwrap();
    let full = answer(&request, &config());
    assert_eq!(
        codec::reply(&full, Command::ReadConfig).unwrap(),
        &config()[..]
    );
    assert!(codec::reply(&full[..8 + 127], Command::ReadConfig).is_err());
    assert!(codec::reply(&captured, Command::ReadConfig).is_err());
    let mut foreign = full.clone();
    foreign[0] = 5;
    assert!(codec::reply(&foreign, Command::ReadConfig).is_err());
    let mut stale = full;
    stale[1] = 0x82;
    assert!(codec::reply(&stale, Command::ReadConfig).is_err());
}

#[test]
fn solid_changes_only_the_static_effect() {
    let original = Config::parse(&config()).unwrap();
    let solid = original.solid();
    let (before, after) = (original.bytes(), solid.bytes());
    assert_eq!(
        (after[0x09], after[0x0a], after[0x3a], after[0x3b]),
        (0, 1, 2, 0x40)
    );
    for i in (0..CONFIG_BYTES).filter(|i| ![0x0a, 0x3b].contains(i)) {
        assert_eq!(before[i], after[i], "byte {i:#x}");
    }
    let mut bright = config();
    bright[0x3b] = 0x26;
    assert_eq!(Config::parse(&bright).unwrap().solid().bytes()[0x3b], 0x20);

    let table = ColorTable::parse(&colors()).unwrap();
    let changed = table.with_static_color([10, 20, 30]);
    assert_eq!(changed.static_color(), [10, 20, 30]);
    assert_eq!(changed.bytes()[24..], table.bytes()[24..]);
    assert_eq!(changed.bytes()[..21], table.bytes()[..21]);
}

#[test]
fn unknown_blocks_are_refused() {
    let mut effect = config();
    effect[0x0a] = 22;
    assert!(Config::parse(&effect).is_err());
    let mut level = config();
    level[0x3b] = 0x50;
    assert!(Config::parse(&level).is_err());
    assert!(Config::parse(&config()[..127]).is_err());
    let mut unsigned = colors();
    unsigned[0x1fb] = 0;
    assert!(ColorTable::parse(&unsigned).is_err());
}

#[test]
fn admits_only_the_520_byte_collection_of_the_captured_keyboard() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/probe-by-tech-gpwb2401-wired.json");
    let report: Report = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let admitted: Vec<_> = report
        .interfaces
        .iter()
        .map(|i| i.endpoint())
        .filter(admits)
        .collect();
    assert_eq!(admitted.len(), 1);
    assert_eq!(admitted[0].reports.map(|r| r.feature), Some(520));
    let mut foreign = endpoint();
    foreign.vid = 0x0c45;
    assert!(!admits(&foreign));
    let mut radio = endpoint();
    (radio.vid, radio.pid) = (0x3554, 0xfa09);
    assert!(!admits(&radio));
}

#[test]
fn discovery_only_reads_and_a_foreign_password_stops_it() {
    let (fake, state) = keyboard(true);
    let bus: Arc<dyn Bus> = fake.clone();
    let found = ByTechDriver.discover(&bus);
    assert_eq!(found.devices.len(), 1, "{:?}", found.notes);
    assert_eq!(opcodes(&fake.log()), [0x82, 0x84, 0x8a, 0x87]);

    state.lock().unwrap().password = [3, 0, 0, 0, 1, 0x99];
    let before = fake.log().len();
    let found = ByTechDriver.discover(&bus);
    assert!(found.devices.is_empty());
    assert!(matches!(found.notes[0].error, DriverError::Rejected(_)));
    assert_eq!(opcodes(&fake.log()[before..]), [0x82]);
}

#[test]
fn backup_precedes_the_write_and_restore_brings_the_original_back() {
    let (fake, state) = keyboard(true);
    let bus: Arc<dyn Bus> = fake.clone();
    let mut device = ByTechDriver.discover(&bus).devices.remove(0);
    let lighting = device.lighting().unwrap();
    assert_eq!(
        lighting.apply_solid(Rgb::new(10, 20, 30)).unwrap(),
        Outcome::Confirmed
    );
    let log = fake.log();
    let first_write = log
        .iter()
        .position(|p| matches!(p, Packet::Feature(p) if p[1] & 0x80 == 0))
        .unwrap();
    assert!(fake.packets_before_backup().unwrap() <= first_write);
    assert_eq!(fake.backups()[0].0, "by-tech");
    {
        let k = state.lock().unwrap();
        assert_eq!(k.config[0x0a], 1);
        assert_eq!(&k.colors[21..24], &[10, 20, 30]);
    }

    assert_eq!(lighting.restore().unwrap(), Outcome::Restored);
    {
        let k = state.lock().unwrap();
        assert_eq!((k.config.clone(), k.colors.clone()), (config(), colors()));
    }
    let snapshot = Snapshot::from_bytes(fake.backups()[0].1.clone()).unwrap();
    assert_eq!(
        lighting.restore_snapshot(&snapshot).unwrap(),
        Outcome::Restored
    );
}

#[test]
fn a_write_the_keyboard_did_not_keep_is_reported_and_not_repeated() {
    let (fake, _) = keyboard(false);
    let bus: Arc<dyn Bus> = fake.clone();
    let mut device = ByTechDriver.discover(&bus).devices.remove(0);
    let error = device
        .lighting()
        .unwrap()
        .apply_solid(Rgb::new(1, 1, 1))
        .unwrap_err();
    assert_eq!(error, DriverError::ReadBackMismatch);
    let writes = opcodes(&fake.log())
        .into_iter()
        .filter(|op| op & 0x80 == 0)
        .count();
    assert_eq!(writes, 2);
}
