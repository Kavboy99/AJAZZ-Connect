use super::*;
use std::path::Path;

fn local_vendor_fixture() -> Option<std::path::PathBuf> {
    // Vendor binaries are not committed; the test runs where a local copy exists.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sonix-136.exe");
    path.exists().then_some(path)
}

#[test]
fn malformed_inputs_do_not_become_images() {
    for data in [
        b"MZ".as_slice(),
        b"<!DOCTYPE html>",
        b"PK\x03\x04",
        b":00000001FF",
    ] {
        assert!(inspect_bytes(data).is_err());
    }
    assert!(inspect_bytes(&[0; 64]).is_err());
}

#[test]
fn raw_image_has_no_automatic_compatibility() {
    let mut data = vec![0; 64];
    data[..4].copy_from_slice(&0x2000_1000u32.to_le_bytes());
    data[4..8].copy_from_slice(&9u32.to_le_bytes());
    let parsed = inspect_bytes(&data).unwrap();
    assert!(parsed.image.plausible_vectors);
    assert!(!parsed.compatibility_proven && !parsed.vendor_code_executed);
}

#[test]
fn parses_original_active_resource_not_template() {
    let Some(path) = local_vendor_fixture() else {
        return;
    };
    let report = inspect_file(&path).unwrap();
    assert_eq!(
        report.image.sha256,
        "58db8a73ea8be7b8f730d67177f3ad78ec1b28b2b73ac15861e67963a1b116ff"
    );
    assert_eq!(report.image.pid, Some(0x80a0));
    assert_eq!(report.advertised_checksum_matches, Some(true));
    assert_eq!(report.metadata.get(&4010).unwrap(), "SN34F280");
}

#[test]
fn inspection_rejects_truncated_resources() {
    let Some(path) = local_vendor_fixture() else {
        return;
    };
    let data = std::fs::read(path).unwrap();
    for n in [64, 512, 4096, data.len() / 2] {
        assert!(inspect_bytes(&data[..n]).is_err());
    }
}
