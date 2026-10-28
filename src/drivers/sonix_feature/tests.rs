use super::codec::{self, FINISH, SAVE, SELECT, START};
use super::*;
use crate::core::{DriverError, ReportSizes};

fn endpoint() -> Endpoint {
    Endpoint::hid(0x0c45, 0x8009, 0xff13, 1)
        .with_product("AK820 MAX")
        .with_reports(Some(ReportSizes {
            input: 0,
            output: 0,
            feature: 65,
        }))
}

#[test]
fn matches_published_packet_fixture_and_rejects_stale_ack() {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("research/sources/BakulBd__OpenKey/fixtures/rgb/sonix-feature-lighting.json");
    let source: serde_json::Value = serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap();
    let actual = [
        codec::command(START, 0).unwrap(),
        codec::command(SELECT, 1).unwrap(),
        codec::solid([255, 0, 0]),
        codec::command(SAVE, 0).unwrap(),
        codec::command(FINISH, 0).unwrap(),
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
    ack[1..5].copy_from_slice(&[4, START, 0, 1]);
    assert!(codec::ack(&ack, START).is_ok());
    assert!(codec::ack(&ack, SELECT).is_err());
    ack[4] = 0xff;
    assert!(codec::ack(&ack, START).is_err());
    for cmd in [0x11, 0x17, 0x20, 0x23, 0x72, 0x80] {
        assert!(codec::command(cmd, 0).is_err());
    }
}

#[test]
fn does_not_commit_or_retry_failed_data() {
    let mut seen = Vec::new();
    let result = codec::apply_transaction(
        |p, handshake| {
            seen.push((p[1], p[2], handshake));
            if p[1] == 1 {
                return Err(DriverError::Io("simulated failed data transfer".into()));
            }
            Ok(())
        },
        [10, 20, 30],
    );
    assert!(result.is_err());
    assert_eq!(
        seen,
        vec![
            (4, START, true),
            (4, SELECT, true),
            (1, 10, false),
            (4, FINISH, false)
        ]
    );
    assert!(
        !seen
            .iter()
            .any(|(prefix, op, _)| *prefix == 4 && *op == SAVE)
    );
}

#[test]
fn admission_requires_exact_product_and_feature_size() {
    assert!(admits(&endpoint()));
    assert!(!admits(&endpoint().with_product("AK820 PRO")));
    let mut other_pid = endpoint();
    other_pid.pid = 0x80a0;
    assert!(!admits(&other_pid));
    assert!(!admits(&endpoint().with_reports(None)));
}
