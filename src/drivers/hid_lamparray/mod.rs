//! HID LampArray (Lighting and Illumination page 0x59): the vendor-neutral standard behind Windows
//! Dynamic Lighting. Colors are not persistent; the device returns to its own program on release.

pub mod codec;
pub mod descriptor;
mod driver;
#[cfg(test)]
mod tests;

use crate::core::Endpoint;

pub use driver::LampArrayDriver;

pub const ID: &str = "hid_lamparray";

fn admits(e: &Endpoint) -> bool {
    e.usage_page == 0x59 && e.usage == 1
}
