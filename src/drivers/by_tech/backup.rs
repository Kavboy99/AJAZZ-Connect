use super::state::ColorTable;
use super::state::Config;
use crate::core::DriverError;
use serde::{Deserialize, Serialize};

const SCHEMA: u32 = 1;
const KIND: &str = "by-tech-lighting";

/// Both lighting blocks as read before the first change. Written to disk before that change.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Backup {
    schema: u32,
    kind: String,
    created_utc: String,
    device_identity: String,
    protocol_identity: String,
    config: Vec<u8>,
    colors: Vec<u8>,
}

impl Backup {
    pub(super) fn from_raw(device: &str, protocol: &str, config: Vec<u8>, colors: Vec<u8>) -> Self {
        Self {
            schema: SCHEMA,
            kind: KIND.into(),
            created_utc: chrono::Utc::now().to_rfc3339(),
            device_identity: device.into(),
            protocol_identity: protocol.into(),
            config,
            colors,
        }
    }
    pub(super) fn recovery_config(
        &self,
        device: &str,
        protocol: &str,
    ) -> Result<Config, DriverError> {
        if self.schema != SCHEMA
            || self.kind != KIND
            || self.device_identity != device
            || self.protocol_identity != protocol
        {
            return Err(DriverError::ForeignBackup);
        }
        let config = Config::parse(&self.config).map_err(|_| DriverError::ForeignBackup)?;
        if self.config[126..] != [0x5a, 0xa5] {
            return Err(DriverError::ForeignBackup);
        }
        Ok(config)
    }
    pub fn new(
        device_identity: &str,
        protocol_identity: &str,
        config: &Config,
        colors: &ColorTable,
    ) -> Self {
        Self {
            schema: SCHEMA,
            kind: KIND.into(),
            created_utc: chrono::Utc::now().to_rfc3339(),
            device_identity: device_identity.into(),
            protocol_identity: protocol_identity.into(),
            config: config.bytes().to_vec(),
            colors: colors.bytes().to_vec(),
        }
    }

    pub fn to_json(&self) -> Result<Vec<u8>, DriverError> {
        serde_json::to_vec_pretty(self).map_err(|e| DriverError::Io(e.to_string()))
    }
}
