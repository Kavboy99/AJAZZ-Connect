use crate::diagnostics;
use crate::drivers;
use crate::storage::atomic;
use crate::transport::SystemBus;
use anyhow::Result;
use std::path::Path;

/// `--probe REPORT.json`: the passive diagnostic report.
pub fn run(out: &Path) -> Result<()> {
    let bus = SystemBus::new();
    if let Some(error) = bus.hid_error() {
        anyhow::bail!("HID enumeration failed: {error}");
    }
    let report = diagnostics::probe(&bus, &drivers::registry());
    atomic::write_json(out, &report)?;
    Ok(())
}
