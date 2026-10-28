//! Pure AA55 packet building and parsing. No IO. Only GET_INFO, GET_LED_EFFECT and
//! SET_LED_EFFECT exist here, all at address zero: no firmware hooks, ISP or memory writer.

use super::info::KeyboardInfo;
use crate::core::{DriverError, sha256_hex};

pub const GET_INFO: u8 = 0x10;
pub const GET_LED_EFFECT: u8 = 0x13;
pub const SET_LED_EFFECT: u8 = 0x23;
pub const INFO_BYTES: usize = 48;
pub const LED_BYTES: usize = 16;
/// Output and input reports, Report ID included.
pub const REPORT_BYTES: usize = 65;

pub fn payload_size(command: u8) -> usize {
    if command == GET_INFO {
        INFO_BYTES
    } else {
        LED_BYTES
    }
}

pub fn packet(command: u8, payload: Option<&[u8]>) -> Result<[u8; REPORT_BYTES], DriverError> {
    if !matches!(command, GET_INFO | GET_LED_EFFECT | SET_LED_EFFECT) {
        return Err(DriverError::rejected(format!(
            "command {command:02X} is not allowed"
        )));
    }
    let size = payload_size(command);
    if (command == SET_LED_EFFECT) != payload.is_some() {
        return Err(DriverError::rejected("GET/SET payload mismatch"));
    }
    let mut p = [0; REPORT_BYTES];
    p[1..9].copy_from_slice(&[0xaa, command, size as u8, 0, 0, 0, 1, 0]);
    if let Some(data) = payload {
        if data.len() != size {
            return Err(DriverError::rejected("lighting payload length"));
        }
        p[9..9 + size].copy_from_slice(data);
    }
    Ok(p)
}

/// The payload of a reply to `command`, with or without the leading Report ID byte.
pub fn response(raw: &[u8], command: u8, size: usize) -> Result<Vec<u8>, DriverError> {
    let p = raw.strip_prefix(&[0]).unwrap_or(raw);
    if size > 56 {
        return Err(DriverError::protocol("reply larger than a report"));
    }
    if p.len() != 64 || p[..5] != [0x55, command, size as u8, 0, 0] {
        return Err(DriverError::protocol("reply does not echo the request"));
    }
    Ok(p[8..8 + size].to_vec())
}

/// Accepts only the LED_EFFECT layout that was researched; anything else is refused unmodified.
pub fn validate_lighting(b: &[u8]) -> Result<(), DriverError> {
    let known = b.len() == LED_BYTES
        && b[0] <= 20
        && b[8] <= 1
        && b[9] <= 5
        && b[10] <= 5
        && b[11] <= 3
        && b[14..] == [0xaa, 0x55];
    if known {
        Ok(())
    } else {
        Err(DriverError::protocol(
            "LED_EFFECT layout differs from the researched protocol",
        ))
    }
}

/// Static color over the current effect, keeping every byte the protocol does not explain.
pub fn solid_payload(original: &[u8], rgb: [u8; 3]) -> Result<Vec<u8>, DriverError> {
    validate_lighting(original)?;
    let mut p = original.to_vec();
    p[0] = 1;
    p[1..4].copy_from_slice(&rgb);
    p[4] = 255;
    p[8] = 0;
    if p[9] == 0 {
        // The user explicitly asked for visible static lighting.
        p[9] = 5;
    }
    Ok(p)
}

/// The vendor's SET representation of the same state.
pub fn wire_form(p: &[u8]) -> Vec<u8> {
    let mut wire = p.to_vec();
    if wire[0] == 20 {
        // Saved per-key mode is written as 128.
        wire[0] = 128;
    }
    wire[4] = 255;
    wire
}

/// Read-back comparison; brightness and the trailer are rewritten by the firmware.
pub fn same_state(a: &[u8], b: &[u8]) -> bool {
    (0..LED_BYTES)
        .filter(|i| ![4, 14, 15].contains(i))
        .all(|i| a.get(i) == b.get(i))
}

/// Firmware 1.30/1.36 on 80A0 answers GET with its factory palette instead of the stored color.
/// The mode must still match exactly.
pub fn palette_quirk(info: &KeyboardInfo, written: &[u8], observed: &[u8]) -> bool {
    if info.pid != 0x80a0
        || !matches!(info.version.as_str(), "1.30" | "1.36")
        || written[0] > 19
        || observed[1..4] != [255, 255, 255]
        || observed[8] != 1
    {
        return false;
    }
    let mut normalized = written.to_vec();
    normalized[1..4].copy_from_slice(&observed[1..4]);
    normalized[8] = observed[8];
    same_state(observed, &normalized)
}

/// Identifies the device and firmware, but not volatile fields such as battery level.
pub fn identity(info: &[u8]) -> Result<String, DriverError> {
    KeyboardInfo::parse(info)?;
    let mut b = info[4..16].to_vec();
    b.push(info[19]);
    b.extend_from_slice(&info[30..33]);
    Ok(sha256_hex(&b))
}
