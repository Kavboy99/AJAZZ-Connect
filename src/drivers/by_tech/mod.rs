//! BY Tech (BYCOMBO4) keyboards on the `258A:010C` platform, wired connection only. The protocol
//! is restored by static analysis of the vendor package; it is experimental until the owner of
//! the keyboard confirms a write (`docs/DEVICES.md`, section 1.1). Settings persist in the keyboard.

mod backup;
pub mod codec;
mod driver;
pub mod state;
#[cfg(test)]
mod tests;

use crate::core::{Detail, DeviceDescriptor, DeviceKind, DriverInfo, Endpoint};
use crate::profiles;

pub use driver::ByTechDriver;

pub const ID: &str = "by_tech";

/// The vendor's `FindHIDDevice` for `Fw = 24` accepts exactly this collection: vendor page
/// `FF00`, usage 1, a 520-byte Feature report. The keyboard's other `FF00` collections (input 4,
/// feature 6) belong to other commands and are never opened.
fn admits(e: &Endpoint) -> bool {
    e.usage_page == 0xff00
        && e.usage == 1
        && e.has_feature(codec::REPORT_BYTES as u16)
        && profiles::find(e, ID).is_some()
}

fn descriptor(e: &Endpoint, driver: DriverInfo) -> DeviceDescriptor {
    let profile = profiles::find(e, ID);
    DeviceDescriptor {
        kind: profile.map_or(DeviceKind::Keyboard, |p| p.kind),
        vendor: profile.map(|p| p.vendor.clone()),
        // The USB string is the generic "Gaming Keyboard"; the profile names the model.
        name: profile.map_or_else(|| e.product.trim().to_owned(), |p| p.model.clone()),
        driver,
        details: vec![
            Detail::Usb {
                vid: e.vid,
                pid: e.pid,
            },
            Detail::Release(e.release),
        ],
        visual: profile.and_then(|p| p.visual.clone()),
    }
}
