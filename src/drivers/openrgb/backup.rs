use super::wire::SdkDevice;
use crate::core::DriverError;
use serde::{Deserialize, Serialize};

const SCHEMA: u32 = 1;

/// The mode and colors the server reported before the first change.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Backup {
    schema: u32,
    created_utc: String,
    device_identity: String,
    device_name: String,
    protocol: u32,
    pub mode_index: u32,
    pub mode_wire: Vec<u8>,
    pub colors: Vec<[u8; 4]>,
}

impl Backup {
    pub fn of(device: &SdkDevice, protocol: u32) -> Option<Self> {
        let mode = device.modes.get(device.active as usize)?;
        Some(Self {
            schema: SCHEMA,
            created_utc: chrono::Utc::now().to_rfc3339(),
            device_identity: device.identity.clone(),
            device_name: device.name.clone(),
            protocol,
            mode_index: device.active,
            mode_wire: mode.wire.clone(),
            colors: device.colors.clone(),
        })
    }

    pub fn check(&self, device: &SdkDevice, protocol: u32) -> Result<(), DriverError> {
        let ours = self.schema == SCHEMA
            && self.protocol == protocol
            && self.device_identity == device.identity
            && self.colors.len() == device.led_count
            && (self.mode_index as usize) < device.modes.len();
        if ours {
            Ok(())
        } else {
            Err(DriverError::ForeignBackup)
        }
    }

    pub fn to_json(&self) -> Result<Vec<u8>, DriverError> {
        serde_json::to_vec_pretty(self).map_err(|e| DriverError::Io(e.to_string()))
    }
}
