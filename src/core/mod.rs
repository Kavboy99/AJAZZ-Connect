//! Shared types and contracts. No IO and no brand names live here: drivers, the application state
//! and the interface all meet on these types.

mod bus;
mod capability;
mod color;
mod device;
mod digest;
mod driver;
mod endpoint;
mod error;
mod snapshot;

pub use bus::{Bus, Guard, HidChannel, Stream};
pub use capability::{Lighting, LightingTraits, Outcome, Rollback};
pub use color::Rgb;
pub use device::{Detail, DeviceDescriptor, DeviceKind};
pub use digest::sha256_hex;
pub use driver::{
    Device, Discovery, Driver, DriverInfo, Identification, Maturity, Note, Requirement,
};
pub use endpoint::{Endpoint, ReportSizes};
pub use error::DriverError;
pub use snapshot::Snapshot;
