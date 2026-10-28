use super::codec;
use super::descriptor::parse_descriptor;
use crate::core::DeviceKind;

const DESCRIPTOR: &[u8] =
    include_bytes!("../../../tests/fixtures/microsoft-lamparray-descriptor.bin");

#[test]
fn microsoft_reference_descriptor_and_variable_report_ids() {
    let m = parse_descriptor(DESCRIPTOR).unwrap();
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
    let other = parse_descriptor(&changed).unwrap();
    assert_eq!((other.attributes, other.multi, other.control), (31, 34, 36));
}

#[test]
fn descriptor_rejects_truncation_wrong_fields_and_shared_ids() {
    for n in 0..DESCRIPTOR.len() {
        assert!(parse_descriptor(&DESCRIPTOR[..n]).is_err(), "{n}");
    }
    let mut wrong = DESCRIPTOR.to_vec();
    let at = wrong.windows(2).position(|x| x == [0x09, 0x71]).unwrap();
    wrong[at + 1] = 0x72;
    assert!(parse_descriptor(&wrong).is_err());
    let mut shared = DESCRIPTOR.to_vec();
    let at = shared.windows(2).position(|x| x == [0x85, 6]).unwrap();
    shared[at + 1] = 5;
    assert!(parse_descriptor(&shared).is_err());
}

#[test]
fn multiupdate_uses_capacity_not_count_for_color_offset() {
    let map = parse_descriptor(DESCRIPTOR).unwrap();
    let p = codec::multi_packet(
        &map,
        &[(0x123, [10, 20, 30, 255]), (7, [40, 50, 60, 255])],
        true,
    )
    .unwrap();
    assert_eq!(p.len(), 51);
    assert_eq!(&p[..7], &[4, 2, 1, 0x23, 1, 7, 0]);
    assert_eq!(&p[19..27], &[10, 20, 30, 255, 40, 50, 60, 255]);
    assert!(p[27..].iter().all(|b| *b == 0));
    assert!(codec::multi_packet(&map, &[], true).is_err());
}

#[test]
fn attributes_map_kind_and_reject_odd_values() {
    let mut a = [0u8; 22];
    a[0] = 104;
    a[14] = 2;
    a[18..22].copy_from_slice(&10_000u32.to_le_bytes());
    let parsed = codec::parse_attributes(&a).unwrap();
    assert_eq!((parsed.lamps, parsed.kind), (104, DeviceKind::Mouse));
    a[14] = 42;
    assert!(codec::parse_attributes(&a).is_err());
    a[14] = 1;
    a[0] = 0;
    assert!(codec::parse_attributes(&a).is_err());
    assert!(codec::parse_attributes(&a[..21]).is_err());
}

#[test]
fn levels_scale_to_the_channel_resolution() {
    assert_eq!(codec::level(200, 0), 200);
    assert_eq!(codec::level(200, 1), 1);
    assert_eq!(codec::level(127, 1), 0);
    assert_eq!(codec::level(255, 16), 15);
    assert_eq!(codec::level(0, 16), 0);
}
