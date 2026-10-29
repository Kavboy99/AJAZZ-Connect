use super::{Bus, DeviceDescriptor, DriverError, Endpoint, Lighting};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Maturity {
    /// Verified on hardware by its owner.
    Stable,
    /// Implemented from a description; off unless the user enables experimental drivers.
    Experimental,
}

/// Something outside the application the driver depends on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Requirement {
    LocalService {
        name: &'static str,
        address: &'static str,
    },
}

/// A driver describes itself. Named after the protocol, not the brand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub maturity: Maturity,
    pub requirement: Option<Requirement>,
}

/// Why one candidate (or the whole driver) produced no device.
#[derive(Clone, Debug)]
pub struct Note {
    /// The product name of the rejected candidate; `None` when the driver failed as a whole.
    pub subject: Option<String>,
    pub error: DriverError,
}

#[derive(Default)]
pub struct Discovery {
    pub devices: Vec<Box<dyn Device>>,
    pub notes: Vec<Note>,
}

/// Read-only answer of a device to a driver's identification request, for diagnostic reports.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Identification {
    pub driver: String,
    pub firmware_version: Option<String>,
    pub reported_vid: Option<u16>,
    pub reported_pid: Option<u16>,
    /// The parsed reply, in the driver's own shape.
    pub details: serde_json::Value,
    pub payload_hex: String,
}

/// A device family: finds devices and nothing else.
pub trait Driver: Send + Sync {
    fn info(&self) -> DriverInfo;

    fn supports_configuration_backup(&self) -> bool {
        false
    }

    /// Static native-program metadata only. Must not open a transport or discover devices.
    fn lighting_preview(&self) -> Option<super::LightingPreview> {
        None
    }

    /// Passive admission by identifiers and report sizes only. No IO.
    fn admits(&self, endpoint: &Endpoint) -> bool;

    /// Read-only. Requests sent here only confirm the family; they never change the device.
    fn discover(&self, bus: &Arc<dyn Bus>) -> Discovery;

    /// Read-only identification of an admitted endpoint for the diagnostic report. `None` when
    /// the protocol has no identification request.
    fn identify(
        &self,
        bus: &dyn Bus,
        endpoint: &Endpoint,
    ) -> Option<Result<Identification, DriverError>>;
}

/// A found and identified device: a passport plus a set of capabilities.
pub trait Device: Send {
    fn descriptor(&self) -> &DeviceDescriptor;

    fn configuration_backup(&mut self) -> Option<&mut dyn super::ConfigurationBackup> {
        None
    }

    fn battery(&mut self) -> Option<&mut dyn super::Battery> {
        None
    }

    fn lighting(&mut self) -> Option<&mut dyn Lighting> {
        None
    }
}
