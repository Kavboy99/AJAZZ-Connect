use ak820max_assistant::{aa55, sonix_feature};
use ak820max_assistant::{lamparray, openrgb};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};
const DESCRIPTOR: &[u8] = include_bytes!("fixtures/microsoft-lamparray-descriptor.bin");
const SDK: &[u8] = include_bytes!("fixtures/openrgb-v3-device.bin");

#[test]
fn aa55_allows_only_stock_lighting_commands_and_preserves_unknown_bytes() {
    let p = aa55::packet(0x10, None).unwrap();
    assert_eq!(&p[..9], &[0, 0xaa, 0x10, 48, 0, 0, 0, 1, 0]);
    for cmd in [0x0f, 0x24, 0x4f, 0x80, 0x81, 0x86] {
        assert!(aa55::packet(cmd, None).is_err());
    }
    let mut original = [0; 16];
    original[0] = 7;
    original[5..8].copy_from_slice(&[10, 11, 12]);
    original[9] = 3;
    original[10] = 4;
    original[12] = 0x34;
    original[13] = 0x56;
    original[14] = 0xaa;
    original[15] = 0x55;
    let changed = aa55::solid_payload(&original, [10, 20, 30]).unwrap();
    assert_eq!(&changed[..5], &[1, 10, 20, 30, 255]);
    for i in [5, 6, 7, 9, 10, 11, 12, 13, 14, 15] {
        assert_eq!(changed[i], original[i]);
    }
    let mut ack = [0; 65];
    ack[1..6].copy_from_slice(&[0x55, 0x23, 16, 0, 0]);
    ack[9..25].copy_from_slice(&changed);
    assert_eq!(aa55::response(&ack, 0x23, 16).unwrap(), changed);
    assert!(aa55::response(&ack, 0x13, 16).is_err());
    ack[4] = 1;
    assert!(aa55::response(&ack, 0x23, 16).is_err());
}
#[test]
fn aa55_identity_tracks_firmware_and_profile_but_not_battery() {
    let mut info = [0; 48];
    info[4..10].copy_from_slice(&[0x45, 0xc, 9, 0x80, 7, 1]);
    let base = aa55::identity(&info).unwrap();
    info[17] = 79;
    assert_eq!(aa55::identity(&info).unwrap(), base);
    info[19] = 1;
    assert_ne!(aa55::identity(&info).unwrap(), base);
    info[19] = 0;
    info[8] = 8;
    assert_ne!(aa55::identity(&info).unwrap(), base);
}
#[test]
fn sonix_feature_matches_published_packet_fixture_and_rejects_stale_ack() {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("research/sources/BakulBd__OpenKey/fixtures/rgb/sonix-feature-lighting.json");
    let source: serde_json::Value = serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap();
    let actual = [
        sonix_feature::command(0x18, 0).unwrap(),
        sonix_feature::command(0x13, 1).unwrap(),
        sonix_feature::solid([255, 0, 0]),
        sonix_feature::command(2, 0).unwrap(),
        sonix_feature::command(0xf0, 0).unwrap(),
    ];
    for (i, packet) in actual.iter().enumerate() {
        let bytes: Vec<_> = source["example"]["sequence"][i]["hex"]
            .as_str()
            .unwrap()
            .split_whitespace()
            .map(|x| u8::from_str_radix(x, 16).unwrap())
            .collect();
        assert_eq!(&packet[1..], bytes);
    }
    let mut ack = [0; 65];
    ack[1..5].copy_from_slice(&[4, 0x18, 0, 1]);
    assert!(sonix_feature::ack(&ack, 0x18).is_ok());
    assert!(sonix_feature::ack(&ack, 0x13).is_err());
    ack[4] = 0xff;
    assert!(sonix_feature::ack(&ack, 0x18).is_err());
    for cmd in [0x11, 0x17, 0x20, 0x23, 0x72, 0x80] {
        assert!(sonix_feature::command(cmd, 0).is_err());
    }
}

#[test]
fn sonix_feature_does_not_commit_or_retry_failed_data() {
    let mut seen = Vec::new();
    let result = sonix_feature::apply_transaction(
        |p, handshake| {
            seen.push((p[1], p[2], handshake));
            if p[1] == 1 {
                anyhow::bail!("simulated failed data transfer");
            }
            Ok(())
        },
        [10, 20, 30],
    );
    assert!(result.is_err());
    assert_eq!(
        seen,
        vec![
            (4, 0x18, true),
            (4, 0x13, true),
            (1, 10, false),
            (4, 0xf0, false)
        ]
    );
    assert!(!seen.iter().any(|(prefix, op, _)| *prefix == 4 && *op == 2));
}

#[test]
fn microsoft_reference_descriptor_and_variable_report_ids() {
    let m = lamparray::parse_descriptor(DESCRIPTOR).unwrap();
    assert_eq!(
        (
            m.attributes,
            m.request,
            m.response,
            m.multi,
            m.range,
            m.control,
            m.capacity
        ),
        (1, 2, 3, 4, 5, 6, 8)
    );
    let mut changed = DESCRIPTOR.to_vec();
    for i in 0..changed.len() - 1 {
        if changed[i] == 0x85 {
            changed[i + 1] += 30;
        }
    }
    let other = lamparray::parse_descriptor(&changed).unwrap();
    assert_eq!((other.attributes, other.multi, other.control), (31, 34, 36));
}
#[test]
fn descriptor_rejects_truncation_wrong_fields_and_shared_ids() {
    for n in 0..DESCRIPTOR.len() {
        assert!(
            lamparray::parse_descriptor(&DESCRIPTOR[..n]).is_err(),
            "{n}"
        );
    }
    let mut wrong = DESCRIPTOR.to_vec();
    let at = wrong.windows(2).position(|x| x == [0x09, 0x71]).unwrap();
    wrong[at + 1] = 0x72;
    assert!(lamparray::parse_descriptor(&wrong).is_err());
    let mut shared = DESCRIPTOR.to_vec();
    let at = shared.windows(2).position(|x| x == [0x85, 6]).unwrap();
    shared[at + 1] = 5;
    assert!(lamparray::parse_descriptor(&shared).is_err());
}
#[test]
fn multiupdate_uses_capacity_not_count_for_color_offset() {
    let map = lamparray::parse_descriptor(DESCRIPTOR).unwrap();
    let p = lamparray::multi_packet(
        &map,
        &[(0x123, [10, 20, 30, 255]), (7, [40, 50, 60, 255])],
        true,
    )
    .unwrap();
    assert_eq!(p.len(), 51);
    assert_eq!(&p[..7], &[4, 2, 1, 0x23, 1, 7, 0]);
    assert_eq!(&p[19..27], &[10, 20, 30, 255, 40, 50, 60, 255]);
    assert!(p[27..].iter().all(|b| *b == 0));
    assert!(lamparray::multi_packet(&map, &[], true).is_err());
}
#[test]
fn sdk_reference_fixture_and_private_identifier_hashing() {
    let d = openrgb::parse_device(SDK, 3, 0).unwrap();
    assert_eq!((d.kind, d.led_count, d.active), (5, 2, 1));
    assert_eq!(d.direct(), Some(0));
    assert!(d.writable());
    assert_eq!(d.colors, vec![[1, 2, 3, 0], [4, 5, 6, 0]]);
    assert_eq!(d.identity.len(), 64);
    assert!(!format!("{d:?}").contains("PRIVATE-"));
    for n in 0..SDK.len() {
        assert!(openrgb::parse_device(&SDK[..n], 3, 0).is_err(), "{n}");
    }
    let mut wrong = SDK.to_vec();
    wrong[8..10].copy_from_slice(&u16::MAX.to_le_bytes());
    assert!(openrgb::parse_device(&wrong, 3, 0).is_err());
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
fn response(s: &mut TcpStream, device: u32, id: u32, p: &[u8]) {
    let mut bytes = b"ORGB".to_vec();
    for n in [device, id, p.len() as u32] {
        bytes.extend_from_slice(&n.to_le_bytes());
    }
    bytes.extend_from_slice(p);
    // TCP fragmentation is intentional: no implementation can assume one read == one packet.
    for part in bytes.chunks(7) {
        s.write_all(part).unwrap();
    }
}
fn expect(s: &mut TcpStream, id: u32, p: &[u8]) {
    let (device, actual, data) = read_frame(s);
    assert_eq!(device, 0);
    assert_eq!(actual, id);
    assert_eq!(data, p);
}
fn count(s: &mut TcpStream) {
    expect(s, 0, &[]);
    response(s, 0, 0, &1u32.to_le_bytes());
}
fn data(s: &mut TcpStream, bytes: &[u8]) {
    expect(s, 1, &3u32.to_le_bytes());
    response(s, 0, 1, bytes);
}
fn handshake(s: &mut TcpStream) {
    expect(s, 40, &3u32.to_le_bytes());
    response(s, 0, 40, &6u32.to_le_bytes());
    expect(s, 50, b"AJAZZ Connect\0");
    count(s);
    data(s, SDK);
}
#[test]
fn sdk_full_color_backup_restore_over_fragmented_tcp() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let worker = thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        handshake(&mut s);
        count(&mut s);
        data(&mut s, SDK);
        let mut direct = SDK[115..174].to_vec();
        // Published mode fields: last six bytes are color_mode(u32), num_colors(u16).
        let n = direct.len();
        direct[n - 6..n - 2].copy_from_slice(&1u32.to_le_bytes());
        let mut mode = Vec::new();
        mode.extend_from_slice(&((8 + direct.len()) as u32).to_le_bytes());
        mode.extend_from_slice(&0u32.to_le_bytes());
        mode.extend_from_slice(&direct);
        expect(&mut s, 1101, &mode);
        let mut colors = 14u32.to_le_bytes().to_vec();
        colors.extend_from_slice(&2u16.to_le_bytes());
        colors.extend_from_slice(&[10, 20, 30, 0, 10, 20, 30, 0]);
        expect(&mut s, 1050, &colors);
        let mut changed = SDK.to_vec();
        changed[111..115].copy_from_slice(&0u32.to_le_bytes());
        changed[295..303].copy_from_slice(&[10, 20, 30, 0, 10, 20, 30, 0]);
        data(&mut s, &changed);
        count(&mut s);
        data(&mut s, &changed);
        let mut old = 14u32.to_le_bytes().to_vec();
        old.extend_from_slice(&2u16.to_le_bytes());
        old.extend_from_slice(&SDK[295..303]);
        expect(&mut s, 1050, &old);
        let mut restore = 74u32.to_le_bytes().to_vec();
        restore.extend_from_slice(&1u32.to_le_bytes());
        restore.extend_from_slice(&SDK[174..240]);
        expect(&mut s, 1101, &restore);
        data(&mut s, SDK);
    });
    let mut c = openrgb::Client::connect(port).unwrap();
    c.solid(0, [10, 20, 30]).unwrap();
    assert_eq!(c.backups.len(), 1);
    let b = serde_json::to_string(&c.backups[0]).unwrap();
    assert!(!b.contains("PRIVATE-"));
    c.restore(0).unwrap();
    worker.join().unwrap();
}
#[test]
fn sdk_refuses_reassigned_device_index_before_any_write() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let worker = thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        handshake(&mut s);
        count(&mut s);
        let mut changed = SDK.to_vec();
        changed[10] = b'X';
        data(&mut s, &changed);
        s.set_read_timeout(Some(std::time::Duration::from_millis(200)))
            .unwrap();
        let mut h = [0; 16];
        assert!(s.read_exact(&mut h).is_err());
    });
    let mut c = openrgb::Client::connect(port).unwrap();
    assert!(
        c.solid(0, [1, 2, 3])
            .unwrap_err()
            .to_string()
            .contains("другому устройству")
    );
    assert!(c.backups.is_empty());
    worker.join().unwrap();
}
#[test]
fn sdk_automatic_save_direct_is_not_writable() {
    let mut changed = SDK.to_vec();
    // Direct mode starts at 115: string(9), value(4), flags(4).
    changed[128..132].copy_from_slice(&(32u32 | 512).to_le_bytes());
    let d = openrgb::parse_device(&changed, 3, 0).unwrap();
    assert!(!d.writable());
}
