use crate::app::identity;
use crate::core::{Bus, Detail};
use crate::drivers;
use crate::storage::atomic;
use crate::transport::SystemBus;
use anyhow::{Context, Result};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;

/// `--lighting-probe DRIVER REPORT.json`: runs one driver's read-only discovery and records what it
/// found. No color is written.
pub fn run(driver_id: &str, out: &Path) -> Result<()> {
    let registry = drivers::registry();
    let driver = registry
        .iter()
        .find(|d| d.info().id == driver_id)
        .with_context(|| {
            let ids: Vec<_> = registry.iter().map(|d| d.info().id).collect();
            format!("unknown driver {driver_id}; known: {}", ids.join(", "))
        })?;
    let bus: Arc<dyn Bus> = Arc::new(SystemBus::new());
    let mut found = driver.discover(&bus);
    let devices: Vec<_> = found
        .devices
        .iter_mut()
        .map(|d| {
            let descriptor = d.descriptor().clone();
            let details: Vec<String> = descriptor
                .details
                .iter()
                .map(|detail| match detail {
                    Detail::Usb { vid, pid } => format!("usb {vid:04X}:{pid:04X}"),
                    Detail::Firmware(v) => format!("firmware {v}"),
                    Detail::Release(r) => format!("bcdDevice {r:04X}"),
                    Detail::Leds(n) => format!("leds {n}"),
                })
                .collect();
            let lighting = d.lighting().map(|l| l.traits());
            json!({
                "name": descriptor.name,
                "vendor": descriptor.vendor,
                "kind": descriptor.kind,
                "details": details,
                "writable": lighting.map(|t| t.writable),
                "persistent": lighting.map(|t| t.persistent),
            })
        })
        .collect();
    let notes: Vec<_> = found
        .notes
        .iter()
        .map(|n| json!({ "subject": n.subject, "error": n.error.to_string() }))
        .collect();
    let report = json!({
        "driver": driver_id,
        "version": identity::version(),
        "devices": devices,
        "notes": notes,
        "firmware_writes": false,
        "settings_writes": false,
    });
    atomic::write_json(out, &report)?;
    Ok(())
}
