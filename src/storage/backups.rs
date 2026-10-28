use super::atomic;
use crate::app::identity;
use crate::core::Snapshot;
use std::{
    io::{self, Read},
    path::{Path, PathBuf},
};

pub fn directory() -> Option<PathBuf> {
    identity::data_dir().map(|dir| dir.join("backups"))
}

/// `<kind>-<UTC timestamp>-<first 16 hex of the device fingerprint>.json`.
pub fn store(kind: &str, device: &str, json: &[u8]) -> io::Result<()> {
    let directory = directory()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "LOCALAPPDATA is not set"))?;
    let short = device.get(..16).unwrap_or(device);
    let name = format!(
        "{kind}-{}-{short}.json",
        chrono::Utc::now().format("%Y%m%d-%H%M%S-%f")
    );
    atomic::write_bytes(&directory.join(name), json)
}

/// Reads at most one byte over the limit, so a huge file is rejected without loading it.
pub fn load(path: &Path) -> io::Result<Snapshot> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(Snapshot::MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    Snapshot::from_bytes(bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "backup is too large"))
}
