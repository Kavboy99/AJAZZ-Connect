//! Classic SONiX Feature protocol of older AK820/MAX/PLUS boards. Experimental: implemented from
//! published research, not verified on a project device. Settings persist in the keyboard and the
//! original effect cannot be read back.

pub mod codec;
mod driver;
#[cfg(test)]
mod tests;

use crate::core::{Detail, DeviceDescriptor, DeviceKind, DriverInfo, Endpoint};
use crate::profiles;

pub use driver::FeatureDriver;

pub const ID: &str = "sonix_feature";

/// The mutex the vendor configuration software takes while it talks to the keyboard.
const VENDOR_LOCK: &str = "Local\\AjazzStudio-AK820MAX";

fn admits(e: &Endpoint) -> bool {
    e.vid == 0x0c45
        && e.usage_page == 0xff13
        && e.usage == 1
        && e.has_feature(codec::REPORT_BYTES as u16)
        && profiles::find(e, ID).is_some()
}

fn descriptor(e: &Endpoint, driver: DriverInfo) -> DeviceDescriptor {
    let profile = profiles::find(e, ID);
    DeviceDescriptor {
        kind: profile.map_or(DeviceKind::Keyboard, |p| p.kind),
        vendor: profile.map(|p| p.vendor.clone()),
        name: e.product.trim().to_owned(),
        driver,
        details: vec![Detail::Usb {
            vid: e.vid,
            pid: e.pid,
        }],
        visual: profile.and_then(|p| p.visual.clone()),
    }
}
