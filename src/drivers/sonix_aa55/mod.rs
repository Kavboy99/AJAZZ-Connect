//! SONiX AA55: the stock configuration protocol of the AK820 Pro/MAX family on SN34F280-class
//! controllers. Lighting settings only; they persist in the keyboard.

mod backup;
pub mod codec;
mod driver;
pub mod info;
#[cfg(test)]
mod tests;

use crate::core::{Detail, DeviceDescriptor, DeviceKind, DriverInfo, Endpoint};
use crate::profiles;

pub use driver::Aa55Driver;

pub const ID: &str = "sonix_aa55";

/// The mutex the vendor configuration software takes while it talks to the keyboard.
const VENDOR_LOCK: &str = "Local\\AjazzStudio-AK820MAX";
const SONIX_VID: u16 = 0x0c45;
/// The SN34F280 ROM bootloader. Never a lighting target, whatever a profile says.
const ROM_BOOTLOADER_PID: u16 = 0x7210;

fn admits(e: &Endpoint) -> bool {
    e.vid == SONIX_VID
        && e.pid != ROM_BOOTLOADER_PID
        && e.usage_page == 0xff68
        && e.usage == 0x61
        && e.has_reports(codec::REPORT_BYTES as u16, codec::REPORT_BYTES as u16)
        && profiles::find(e, ID).is_some()
}

fn descriptor(e: &Endpoint, driver: DriverInfo) -> DeviceDescriptor {
    let profile = profiles::find(e, ID);
    DeviceDescriptor {
        kind: profile.map_or(DeviceKind::Keyboard, |p| p.kind),
        vendor: profile.map(|p| p.vendor.clone()),
        name: if e.product.trim().is_empty() {
            profile.map_or_else(String::new, |p| p.model.clone())
        } else {
            e.product.trim().to_owned()
        },
        driver,
        details: vec![Detail::Usb {
            vid: e.vid,
            pid: e.pid,
        }],
        visual: profile.and_then(|p| p.visual.clone()),
    }
}
