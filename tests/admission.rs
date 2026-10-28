//! Hardware-safety rule 2: a driver for one platform never admits a device of another. Every probe
//! fixture in `tests/fixtures/probe-*.json` is checked against every registered driver.

use lusorium_studio::core::{Endpoint, ReportSizes};
use lusorium_studio::diagnostics::Report;
use lusorium_studio::drivers::registry;

/// Which driver may admit endpoints of each captured device; `None` means no driver may.
const FIXTURES: &[(&str, Option<&str>)] = &[("probe-by-tech-gpwb2401-wired.json", Some("by_tech"))];

fn fixture(name: &str) -> Report {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

#[test]
fn every_probe_fixture_is_listed() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for entry in std::fs::read_dir(dir).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        if name.starts_with("probe-") && name.ends_with(".json") {
            assert!(
                FIXTURES.iter().any(|(f, _)| *f == name),
                "{name} is not checked"
            );
        }
    }
}

#[test]
fn foreign_drivers_reject_every_captured_device() {
    for (name, owner) in FIXTURES {
        let report = fixture(name);
        assert!(!report.interfaces.is_empty(), "{name}");
        for interface in &report.interfaces {
            let endpoint = interface.endpoint();
            for driver in registry() {
                let id = driver.info().id;
                if Some(id) != *owner {
                    assert!(
                        !driver.admits(&endpoint),
                        "{id} admits {name} {:04X}:{:04X}",
                        endpoint.vid,
                        endpoint.pid
                    );
                }
            }
        }
    }
}

fn sizes(input: u16, output: u16, feature: u16) -> Option<ReportSizes> {
    Some(ReportSizes {
        input,
        output,
        feature,
    })
}

/// Synthetic endpoints shaped like the researched families (not captured from hardware). Each must
/// be admitted by exactly its own driver.
#[test]
fn each_family_is_admitted_by_exactly_one_driver() {
    let cases = [
        (
            Endpoint::hid(0x0c45, 0x80a0, 0xff68, 0x61)
                .with_product("AK820 MAX")
                .with_reports(sizes(65, 65, 0)),
            "sonix_aa55",
        ),
        (
            Endpoint::hid(0x0c45, 0x8009, 0xff13, 1)
                .with_product("AK820 MAX")
                .with_reports(sizes(0, 0, 65)),
            "sonix_feature",
        ),
        (
            Endpoint::hid(0x1234, 0x5678, 0x59, 1).with_reports(sizes(0, 0, 23)),
            "hid_lamparray",
        ),
        (
            Endpoint::hid(0x258a, 0x010c, 0xff00, 1)
                .with_product("Gaming Keyboard")
                .with_reports(sizes(8, 0, 520)),
            "by_tech",
        ),
    ];
    for (endpoint, owner) in cases {
        let admitted: Vec<_> = registry()
            .into_iter()
            .filter(|d| d.admits(&endpoint))
            .map(|d| d.info().id)
            .collect();
        assert_eq!(admitted, vec![owner]);
    }
}
