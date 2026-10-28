use super::state::{ColorTable, Config};
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

    /// The blocks, if the backup comes from this port and this keyboard and is well-formed.
    pub fn blocks(
        &self,
        device_identity: &str,
        protocol_identity: &str,
    ) -> Result<(Config, ColorTable), DriverError> {
        let ours = self.schema == SCHEMA
            && self.kind == KIND
            && self.device_identity == device_identity
            && self.protocol_identity == protocol_identity;
        if !ours {
            return Err(DriverError::ForeignBackup);
        }
        let config = Config::parse(&self.config).map_err(|_| DriverError::ForeignBackup)?;
        let colors = ColorTable::parse(&self.colors).map_err(|_| DriverError::ForeignBackup)?;
        Ok((config, colors))
    }

    pub fn to_json(&self) -> Result<Vec<u8>, DriverError> {
        serde_json::to_vec_pretty(self).map_err(|e| DriverError::Io(e.to_string()))
    }
}
