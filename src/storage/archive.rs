//! Bounded, compressed portable snapshots. No archive paths are extracted or executed.
use crate::core::{DriverError, Snapshot, sha256_hex};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema: u32,
    payload_sha256: String,
    payload: Vec<u8>,
}

pub fn encode(snapshot: &Snapshot) -> Result<Vec<u8>, DriverError> {
    let envelope = Envelope {
        schema: 1,
        payload_sha256: sha256_hex(snapshot.bytes()),
        payload: snapshot.bytes().to_vec(),
    };
    let bytes = serde_json::to_vec(&envelope).map_err(|e| DriverError::Io(e.to_string()))?;
    if bytes.len() > Snapshot::MAX_BYTES {
        return Err(DriverError::ForeignBackup);
    }
    let mut gzip = GzEncoder::new(Vec::new(), Compression::default());
    gzip.write_all(&bytes)?;
    Ok(gzip.finish()?)
}

pub fn decode(bytes: &[u8]) -> Result<Snapshot, DriverError> {
    if bytes.len() > Snapshot::MAX_BYTES {
        return Err(DriverError::ForeignBackup);
    }
    if !bytes.starts_with(&[0x1f, 0x8b]) {
        // Historical driver JSON backups are imported without rewriting the originals.
        let _: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|_| DriverError::ForeignBackup)?;
        return Snapshot::from_bytes(bytes.to_vec());
    }
    let mut decoded = Vec::new();
    GzDecoder::new(bytes)
        .take(Snapshot::MAX_BYTES as u64 + 1)
        .read_to_end(&mut decoded)
        .map_err(|_| DriverError::ForeignBackup)?;
    if decoded.len() > Snapshot::MAX_BYTES {
        return Err(DriverError::ForeignBackup);
    }
    let envelope: Envelope =
        serde_json::from_slice(&decoded).map_err(|_| DriverError::ForeignBackup)?;
    if envelope.schema != 1 || sha256_hex(&envelope.payload) != envelope.payload_sha256 {
        return Err(DriverError::ForeignBackup);
    }
    Snapshot::from_bytes(envelope.payload)
}

pub fn load(path: &Path) -> Result<Snapshot, DriverError> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(Snapshot::MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    decode(&bytes)
}

pub fn save(path: &Path, snapshot: &Snapshot) -> Result<(), DriverError> {
    super::atomic::write_bytes(path, &encode(snapshot)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compressed_and_legacy_backups_roundtrip_and_corruption_is_rejected()
    -> Result<(), DriverError> {
        let snapshot = Snapshot::from_bytes(br#"{"kind":"test","config":[1,2,3]}"#.to_vec())?;
        let packed = encode(&snapshot)?;
        assert_eq!(decode(&packed)?.bytes(), snapshot.bytes());
        assert_eq!(decode(snapshot.bytes())?.bytes(), snapshot.bytes());
        let mut damaged = packed.clone();
        damaged[15] ^= 0x80;
        assert!(decode(&damaged).is_err());
        assert!(decode(&packed[..packed.len() - 4]).is_err());
        assert!(decode(&vec![0; Snapshot::MAX_BYTES + 1]).is_err());
        let mut gzip = GzEncoder::new(Vec::new(), Compression::default());
        gzip.write_all(&vec![b' '; Snapshot::MAX_BYTES + 1])?;
        assert!(decode(&gzip.finish()?).is_err());
        let envelope = Envelope {
            schema: 1,
            payload: vec![1, 2],
            payload_sha256: "wrong".into(),
        };
        let mut gzip = GzEncoder::new(Vec::new(), Compression::default());
        gzip.write_all(
            &serde_json::to_vec(&envelope).map_err(|e| DriverError::Io(e.to_string()))?,
        )?;
        assert!(decode(&gzip.finish()?).is_err());
        Ok(())
    }
}
