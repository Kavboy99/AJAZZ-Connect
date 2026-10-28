use super::OpenRgbDriver;
use super::wire::parse_device;
use crate::app::identity;
use crate::core::{Bus, Driver, DriverError, Outcome, Rgb};
use crate::drivers::fake_bus::FakeBus;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

const SDK: &[u8] = include_bytes!("../../../tests/fixtures/openrgb-v3-device.bin");

#[test]
fn reference_fixture_and_private_identifier_hashing() {
    let d = parse_device(SDK, 3, 0).unwrap();
    assert_eq!((d.kind, d.led_count, d.active), (5, 2, 1));
    assert_eq!(d.direct(), Some(0));
    assert!(d.writable());
    assert_eq!(d.colors, vec![[1, 2, 3, 0], [4, 5, 6, 0]]);
    assert_eq!(d.identity.len(), 64);
    assert!(!format!("{d:?}").contains("PRIVATE-"));
    for n in 0..SDK.len() {
        assert!(parse_device(&SDK[..n], 3, 0).is_err(), "{n}");
    }
    let mut wrong = SDK.to_vec();
    wrong[8..10].copy_from_slice(&u16::MAX.to_le_bytes());
    assert!(parse_device(&wrong, 3, 0).is_err());
}

#[test]
fn automatic_save_direct_is_not_writable() {
    let mut changed = SDK.to_vec();
    // Direct mode starts at 115: string(9), value(4), flags(4).
    changed[128..132].copy_from_slice(&(32u32 | 512).to_le_bytes());
    assert!(!parse_device(&changed, 3, 0).unwrap().writable());
}

fn read_frame(s: &mut TcpStream) -> (u32, u32, Vec<u8>) {
    let mut h = [0; 16];
    s.read_exact(&mut h).unwrap();
    assert_eq!(&h[..4], b"ORGB");
    let n = u32::from_le_bytes(h[12..16].try_into().unwrap()) as usize;
    assert!(n < 1_000_000);
    let mut p = vec![0; n];
    s.read_exact(&mut p).unwrap();
    (
        u32::from_le_bytes(h[4..8].try_into().unwrap()),
        u32::from_le_bytes(h[8..12].try_into().unwrap()),
        p,
    )
}

fn respond(s: &mut TcpStream, id: u32, p: &[u8]) {
    let mut bytes = b"ORGB".to_vec();
    for n in [0, id, p.len() as u32] {
        bytes.extend_from_slice(&n.to_le_bytes());
    }
    bytes.extend_from_slice(p);
    // TCP fragmentation is intentional: one read is never one packet.
    for part in bytes.chunks(7) {
        s.write_all(part).unwrap();
    }
}

fn expect(s: &mut TcpStream, id: u32, p: &[u8]) {
    let (device, actual, data) = read_frame(s);
    assert_eq!((device, actual), (0, id));
    assert_eq!(data, p);
}

fn count(s: &mut TcpStream) {
    expect(s, 0, &[]);
    respond(s, 0, &1u32.to_le_bytes());
}

fn data(s: &mut TcpStream, bytes: &[u8]) {
    expect(s, 1, &3u32.to_le_bytes());
    respond(s, 1, bytes);
}

fn handshake(s: &mut TcpStream) {
    expect(s, 40, &3u32.to_le_bytes());
    respond(s, 40, &6u32.to_le_bytes());
    let mut name = identity::CLIENT_NAME.as_bytes().to_vec();
    name.push(0);
    expect(s, 50, &name);
    count(s);
    data(s, SDK);
}

fn server(script: impl FnOnce(&mut TcpStream) + Send + 'static) -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let worker = thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        handshake(&mut s);
        script(&mut s);
    });
    (port, worker)
}

fn no_responder() -> crate::drivers::fake_bus::Responder {
    Box::new(|_| Vec::new())
}

#[test]
fn full_color_backup_restore_over_fragmented_tcp() {
    let (port, worker) = server(|s| {
        count(s);
        data(s, SDK);
        let mut direct = SDK[115..174].to_vec();
        // Published mode fields: the last six bytes are color_mode(u32), num_colors(u16).
        let n = direct.len();
        direct[n - 6..n - 2].copy_from_slice(&1u32.to_le_bytes());
        let mut mode = ((8 + direct.len()) as u32).to_le_bytes().to_vec();
        mode.extend_from_slice(&0u32.to_le_bytes());
        mode.extend_from_slice(&direct);
        expect(s, 1101, &mode);
        let mut colors = 14u32.to_le_bytes().to_vec();
        colors.extend_from_slice(&2u16.to_le_bytes());
        colors.extend_from_slice(&[10, 20, 30, 0, 10, 20, 30, 0]);
        expect(s, 1050, &colors);
        let mut changed = SDK.to_vec();
        changed[111..115].copy_from_slice(&0u32.to_le_bytes());
        changed[295..303].copy_from_slice(&[10, 20, 30, 0, 10, 20, 30, 0]);
        data(s, &changed);
        count(s);
        data(s, &changed);
        let mut old = 14u32.to_le_bytes().to_vec();
        old.extend_from_slice(&2u16.to_le_bytes());
        old.extend_from_slice(&SDK[295..303]);
        expect(s, 1050, &old);
        let mut restore = 74u32.to_le_bytes().to_vec();
        restore.extend_from_slice(&1u32.to_le_bytes());
        restore.extend_from_slice(&SDK[174..240]);
        expect(s, 1101, &restore);
        data(s, SDK);
    });
    let fake = FakeBus::new(Vec::new(), no_responder());
    let bus: Arc<dyn Bus> = fake.clone();
    let mut found = OpenRgbDriver::on_port(port).discover(&bus);
    assert_eq!(found.devices.len(), 1, "{:?}", found.notes);
    let lighting = found.devices[0].lighting().unwrap();
    assert_eq!(
        lighting.apply_solid(Rgb::new(10, 20, 30)),
        Ok(Outcome::ServiceConfirmed)
    );
    let backups = fake.backups();
    assert_eq!(backups.len(), 1);
    assert!(!String::from_utf8_lossy(&backups[0].1).contains("PRIVATE-"));
    assert_eq!(lighting.restore(), Ok(Outcome::Restored));
    worker.join().unwrap();
}

#[test]
fn refuses_reassigned_device_index_before_any_write() {
    let (port, worker) = server(|s| {
        count(s);
        let mut changed = SDK.to_vec();
        changed[10] = b'X';
        data(s, &changed);
        s.set_read_timeout(Some(std::time::Duration::from_millis(200)))
            .unwrap();
        let mut h = [0; 16];
        assert!(s.read_exact(&mut h).is_err());
    });
    let fake = FakeBus::new(Vec::new(), no_responder());
    let bus: Arc<dyn Bus> = fake.clone();
    let mut found = OpenRgbDriver::on_port(port).discover(&bus);
    let lighting = found.devices[0].lighting().unwrap();
    assert_eq!(
        lighting.apply_solid(Rgb::new(1, 2, 3)),
        Err(DriverError::Changed)
    );
    assert!(fake.backups().is_empty());
    worker.join().unwrap();
}
