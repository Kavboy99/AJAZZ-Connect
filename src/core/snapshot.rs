use super::DriverError;
use serde::de::DeserializeOwned;

/// A rollback file as read from disk. Only the driver that wrote it can interpret it, and it
/// checks that the file belongs to the device before using it.
#[derive(Clone, Debug)]
pub struct Snapshot {
    bytes: Vec<u8>,
}

impl Snapshot {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Backups are small JSON files; anything larger is not ours.
    pub const MAX_BYTES: usize = 128 * 1024;

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, DriverError> {
        if bytes.len() > Self::MAX_BYTES {
            return Err(DriverError::ForeignBackup);
        }
        Ok(Self { bytes })
    }

    pub fn parse<T: DeserializeOwned>(&self) -> Result<T, DriverError> {
        serde_json::from_slice(&self.bytes).map_err(|_| DriverError::ForeignBackup)
    }
}
