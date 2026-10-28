use super::InspectError;
use crate::core::sha256_hex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageInfo {
    pub bytes: usize,
    pub sha256: String,
    pub word_sum: Option<String>,
    pub initial_sp: u32,
    pub reset: u32,
    pub plausible_vectors: bool,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    pub version_raw: Option<u16>,
}

pub(super) fn slice(data: &[u8], at: usize, size: usize) -> Result<&[u8], InspectError> {
    at.checked_add(size)
        .and_then(|end| data.get(at..end))
        .ok_or_else(|| InspectError::malformed("truncated file"))
}

pub(super) fn u16_at(data: &[u8], at: usize) -> Result<u16, InspectError> {
    let b = slice(data, at, 2)?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}

pub(super) fn u32_at(data: &[u8], at: usize) -> Result<u32, InspectError> {
    let b = slice(data, at, 4)?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// Cortex-M vector table and the SONiX identity block at 0x9000, when present.
pub(super) fn image_info(data: &[u8]) -> Result<ImageInfo, InspectError> {
    if !(64..=1024 * 1024).contains(&data.len()) {
        return Err(InspectError::malformed("unusual image size"));
    }
    let initial_sp = u32_at(data, 0)?;
    let reset = u32_at(data, 4)?;
    let has_identity = data.get(0x9000..0x9004) == Some(&[0, 0, 0, 0x12]);
    let word_sum = data.len().is_multiple_of(2).then(|| {
        let sum = data
            .as_chunks::<2>()
            .0
            .iter()
            .fold(0u16, |s, p| s.wrapping_add(u16::from_le_bytes(*p)));
        format!("{sum:04X}")
    });
    let identity = |at: usize| -> Result<Option<u16>, InspectError> {
        if has_identity {
            u16_at(data, at).map(Some)
        } else {
            Ok(None)
        }
    };
    Ok(ImageInfo {
        bytes: data.len(),
        sha256: sha256_hex(data),
        word_sum,
        initial_sp,
        reset,
        plausible_vectors: (0x2000_0000..0x2008_0000).contains(&initial_sp)
            && reset & 1 == 1
            && (reset as usize) < data.len(),
        vid: identity(0x9004)?,
        pid: identity(0x9006)?,
        version_raw: identity(0x9008)?,
    })
}
