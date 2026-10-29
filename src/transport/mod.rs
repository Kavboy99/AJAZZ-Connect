//! The only place with `unsafe` and real IO. Everything above talks to devices through
//! `core::Bus`, implemented here by `SystemBus`.

#[cfg(windows)]
pub(crate) mod acrylic;
#[cfg(windows)]
mod hid;
#[cfg(windows)]
mod hid_caps;
mod lease;
mod system;
mod system_bus;
mod tcp;
mod watch;

pub use system::animations_enabled;
pub use system_bus::SystemBus;
pub use watch::{DeviceWatch, OnChange};
