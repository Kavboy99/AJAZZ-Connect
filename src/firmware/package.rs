use super::InspectError;
use super::image::{ImageInfo, image_info};
use super::pe::Pe;
use crate::core::sha256_hex;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Read, path::Path};

const MAX_PACKAGE: u64 = 32 * 1024 * 1024;
/// The SONiX updater keeps the image in RCDATA 4000 and its metadata strings next to it.
const IMAGE_RESOURCE: u32 = 4000;
const METADATA_RESOURCES: [u32; 7] = [4007, 4008, 4009, 4010, 4013, 4014, 4016];
const CHECKSUM_RESOURCE: u32 = 4014;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackageInfo {
    pub kind: String,
    pub sha256: String,
    pub bytes: usize,
    pub image: ImageInfo,
    pub metadata: BTreeMap<u32, String>,
    pub advertised_checksum_matches: Option<bool>,
    pub compatibility_proven: bool,
    pub vendor_code_executed: bool,
}

fn utf16(bytes: &[u8]) -> Result<String, InspectError> {
    if bytes.len() > 1024 || !bytes.len().is_multiple_of(2) {
        return Err(InspectError::malformed("metadata string"));
    }
    let words: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u16::from_le_bytes(*p))
        .collect();
    String::from_utf16(&words)
        .map(|s| s.trim_end_matches('\0').to_owned())
        .map_err(|_| InspectError::malformed("metadata string"))
}

pub fn inspect_bytes(data: &[u8]) -> Result<PackageInfo, InspectError> {
    if data.len() as u64 > MAX_PACKAGE {
        return Err(InspectError::TooLarge);
    }
    let mut metadata = BTreeMap::new();
    let (kind, image) = if data.starts_with(b"MZ") {
        let resources = Pe::new(data)?.rcdata()?;
        let image = resources
            .get(&IMAGE_RESOURCE)
            .ok_or_else(|| InspectError::malformed("no RCDATA 4000 image"))?;
        for id in METADATA_RESOURCES {
            if let Some(bytes) = resources.get(&id) {
                metadata.insert(id, utf16(bytes)?);
            }
        }
        ("SONiX PE / RCDATA4000", image_info(image)?)
    } else {
        if data.starts_with(b":") || data.starts_with(b"<") || data.starts_with(b"PK") {
            return Err(InspectError::NotFirmware);
        }
        let image = image_info(data)?;
        if !image.plausible_vectors {
            return Err(InspectError::NotFirmware);
        }
        ("ARM BIN", image)
    };
    let advertised_checksum_matches = metadata
        .get(&CHECKSUM_RESOURCE)
        .map(|sum| image.word_sum.as_deref() == Some(sum.to_uppercase().as_str()));
    Ok(PackageInfo {
        kind: kind.into(),
        sha256: sha256_hex(data),
        bytes: data.len(),
        image,
        metadata,
        advertised_checksum_matches,
        compatibility_proven: false,
        vendor_code_executed: false,
    })
}

pub fn inspect_file(path: &Path) -> Result<PackageInfo, InspectError> {
    let io = |e: std::io::Error| InspectError::Io(e.to_string());
    let mut data = Vec::new();
    std::fs::File::open(path)
        .map_err(io)?
        .take(MAX_PACKAGE + 1)
        .read_to_end(&mut data)
        .map_err(io)?;
    inspect_bytes(&data)
}
