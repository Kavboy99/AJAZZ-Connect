use crate::core::DriverError;
use serde::{Deserialize, Serialize};

/// The 48-byte `GET_INFO` reply.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyboardInfo {
    pub vid: u16,
    pub pid: u16,
    pub version: String,
    pub manufacturer: u16,
    pub product: u16,
    pub work_mode: u8,
    pub battery: u8,
    pub charging: u8,
    pub profile: u8,
    pub rt_precision: u8,
    pub frame_version: u8,
    pub lighting_version: u8,
    pub firmware_status: u8,
    pub gif_frames: u16,
    pub led_frames: u16,
    pub tft_frames: u16,
}

impl KeyboardInfo {
    pub fn parse(b: &[u8]) -> Result<Self, DriverError> {
        if b.len() != 48 {
            return Err(DriverError::protocol("GET_INFO length"));
        }
        if b[17] > 100 {
            return Err(DriverError::protocol("GET_INFO battery above 100"));
        }
        if b[8] & 15 > 9 || b[8] >> 4 > 9 {
            return Err(DriverError::protocol("GET_INFO version is not BCD"));
        }
        let word = |at: usize| u16::from_le_bytes([b[at], b[at + 1]]);
        Ok(Self {
            vid: word(4),
            pid: word(6),
            version: format!("{}.{:02}", b[9], (b[8] >> 4) * 10 + (b[8] & 15)),
            manufacturer: word(12),
            product: word(14),
            work_mode: b[16],
            battery: b[17],
            charging: b[18],
            profile: b[19],
            rt_precision: b[29],
            frame_version: b[30],
            lighting_version: b[31],
            firmware_status: b[32],
            gif_frames: word(24),
            led_frames: word(26),
            tft_frames: word(22),
        })
    }
}
