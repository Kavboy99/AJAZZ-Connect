use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

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
    pub fn parse(b: &[u8]) -> Result<Self> {
        ensure!(b.len() == 48, "Некорректная длина GET_INFO");
        ensure!(b[17] <= 100, "Некорректное значение заряда");
        ensure!(b[8] & 15 <= 9 && b[8] >> 4 <= 9, "Некорректная BCD-версия");
        Ok(Self {
            vid: u16::from_le_bytes([b[4], b[5]]),
            pid: u16::from_le_bytes([b[6], b[7]]),
            version: format!("{}.{:02}", b[9], (b[8] >> 4) * 10 + (b[8] & 15)),
            manufacturer: u16::from_le_bytes([b[12], b[13]]),
            product: u16::from_le_bytes([b[14], b[15]]),
            work_mode: b[16],
            battery: b[17],
            charging: b[18],
            profile: b[19],
            rt_precision: b[29],
            frame_version: b[30],
            lighting_version: b[31],
            firmware_status: b[32],
            gif_frames: u16::from_le_bytes([b[24], b[25]]),
            led_frames: u16::from_le_bytes([b[26], b[27]]),
            tft_frames: u16::from_le_bytes([b[22], b[23]]),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_known_info_and_rejects_malformed_version() {
        let mut b = [0; 48];
        b[4..10].copy_from_slice(&[0x45, 0x0c, 0xa0, 0x80, 0x36, 1]);
        b[12..16].copy_from_slice(&[0x66, 1, 4, 0x10]);
        let p = KeyboardInfo::parse(&b).unwrap();
        assert_eq!(
            (p.vid, p.pid, p.manufacturer, p.product),
            (0xc45, 0x80a0, 0x166, 0x1004)
        );
        assert_eq!(p.version, "1.36");
        b[8] = 0xfa;
        assert!(KeyboardInfo::parse(&b).is_err());
        assert!(KeyboardInfo::parse(&b[..47]).is_err());
    }
}
