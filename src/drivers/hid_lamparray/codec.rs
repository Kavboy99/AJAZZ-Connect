//! Pure LampArray report building and parsing.

use super::descriptor::ReportMap;
use crate::core::{DeviceKind, DriverError};

pub const ATTRIBUTES_BYTES: usize = 22;
pub const LAMP_ATTRIBUTES_BYTES: usize = 28;

/// `LampArrayAttributesReport` fields the driver relies on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attributes {
    pub lamps: u16,
    pub kind: DeviceKind,
    pub interval_us: u32,
}

fn word(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn dword(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

/// `LampArrayKind` from the HID usage tables.
fn kind(value: u32) -> Option<DeviceKind> {
    Some(match value {
        1 => DeviceKind::Keyboard,
        2 => DeviceKind::Mouse,
        3 => DeviceKind::Gamepad,
        7 => DeviceKind::Component,
        5 | 6 | 9 | 10 => DeviceKind::Lighting,
        4 | 8 => DeviceKind::Other,
        _ => return None,
    })
}

pub fn parse_attributes(b: &[u8]) -> Result<Attributes, DriverError> {
    if b.len() != ATTRIBUTES_BYTES {
        return Err(DriverError::protocol("attributes report length"));
    }
    let lamps = word(b, 0);
    if !(1..=512).contains(&lamps) {
        return Err(DriverError::rejected("unusual lamp count"));
    }
    let kind = kind(dword(b, 14)).ok_or_else(|| DriverError::rejected("unknown LampArrayKind"))?;
    let interval_us = dword(b, 18);
    if interval_us > 1_000_000 {
        return Err(DriverError::rejected("update interval too slow"));
    }
    Ok(Attributes {
        lamps,
        kind,
        interval_us,
    })
}

/// A lamp's per-channel level counts if it is programmable, after checking it answers for `id`.
pub fn parse_lamp(b: &[u8], id: u16) -> Result<Option<[u8; 4]>, DriverError> {
    if b.len() != LAMP_ATTRIBUTES_BYTES || word(b, 0) != id {
        return Err(DriverError::protocol("lamp attributes for another LampId"));
    }
    Ok((b[26] != 0).then(|| [b[22], b[23], b[24], b[25]]))
}

/// Scales an 8-bit value to a channel with `count` levels (0 and 255 mean full 8-bit range).
pub fn level(value: u8, count: u8) -> u8 {
    match count {
        0 | 255 => value,
        1 => u8::from(value >= 128),
        n => ((value as u32 * (n - 1) as u32 + 127) / 255) as u8,
    }
}

pub fn lamp_color(rgb: [u8; 3], levels: [u8; 4]) -> [u8; 4] {
    [
        level(rgb[0], levels[0]),
        level(rgb[1], levels[1]),
        level(rgb[2], levels[2]),
        level(255, levels[3]),
    ]
}

/// Colors sit after `capacity` LampIds, not after the used count.
pub fn multi_packet(
    map: &ReportMap,
    lamps: &[(u16, [u8; 4])],
    complete: bool,
) -> Result<Vec<u8>, DriverError> {
    if lamps.is_empty() || lamps.len() > map.capacity {
        return Err(DriverError::rejected("lamp count does not fit MultiUpdate"));
    }
    let mut p = vec![0; 3 + 6 * map.capacity];
    p[0] = map.multi;
    p[1] = lamps.len() as u8;
    p[2] = u8::from(complete);
    for (i, (id, rgb)) in lamps.iter().enumerate() {
        p[3 + i * 2..5 + i * 2].copy_from_slice(&id.to_le_bytes());
        let at = 3 + map.capacity * 2 + i * 4;
        p[at..at + 4].copy_from_slice(rgb);
    }
    Ok(p)
}

/// `LampArrayControlReport`: autonomous mode on or off.
pub fn control(map: &ReportMap, autonomous: bool) -> [u8; 2] {
    [map.control, u8::from(autonomous)]
}
