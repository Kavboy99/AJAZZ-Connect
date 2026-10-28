use crate::core::DriverError;
use serde::{Deserialize, Serialize};

const SCHEMA: u32 = 1;
const KIND: &str = "aa55-lighting";

/// The LED_EFFECT read before the first change. Written to disk before the write that changes it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Backup {
    schema: u32,
    kind: String,
    created_utc: String,
    device_identity: String,
    protocol_identity: String,
    pub lighting: Vec<u8>,
}

impl Backup {
    pub fn new(device_identity: &str, protocol_identity: &str, lighting: Vec<u8>) -> Self {
        Self {
            schema: SCHEMA,
            kind: KIND.into(),
            created_utc: chrono::Utc::now().to_rfc3339(),
            device_identity: device_identity.into(),
            protocol_identity: protocol_identity.into(),
            lighting,
        }
    }

    /// The backup must come from this port and this firmware.
    pub fn check(&self, device_identity: &str, protocol_identity: &str) -> Result<(), DriverError> {
        let ours = self.schema == SCHEMA
            && self.kind == KIND
            && self.device_identity == device_identity
            && self.protocol_identity == protocol_identity;
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
