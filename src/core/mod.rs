//! Shared types and contracts. No IO and no brand names live here: drivers, the application state
//! and the interface all meet on these types.

mod bus;
mod capability;
mod color;
mod configuration;
mod device;
mod digest;
mod driver;
mod endpoint;
mod error;
mod lighting;
mod snapshot;

pub use bus::{Bus, Guard, HidChannel, Stream};
pub use capability::{Battery, Lighting, LightingTraits, Outcome, Rollback};
pub use color::Rgb;
pub use configuration::{BackupCoverage, BackupInfo, ConfigurationBackup, RestoreScope};
pub use device::{Detail, DeviceDescriptor, DeviceKind, PowerSource};
pub use digest::sha256_hex;
pub use driver::{
    Device, Discovery, Driver, DriverInfo, Identification, Maturity, Note, Requirement,
};
pub use endpoint::{Endpoint, ReportSizes};
pub use error::DriverError;
pub use lighting::{
    LightingEditor, LightingEffect, LightingPreset, LightingPreview, LightingSettings,
};
pub use snapshot::Snapshot;
