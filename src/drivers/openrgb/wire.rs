//! Pure parsing and encoding of the published OpenRGB SDK, protocol levels 1..3.

use crate::core::{DeviceKind, DriverError, sha256_hex};

pub const LIMIT: usize = 4 * 1024 * 1024;
const DIRECT_FLAGS: u32 = 1 << 5;
const AUTO_SAVE: u32 = 1 << 9;

pub const REQUEST_COUNT: u32 = 0;
pub const REQUEST_DEVICE: u32 = 1;
pub const PROTOCOL_VERSION: u32 = 40;
pub const SET_CLIENT_NAME: u32 = 50;
pub const DEVICE_LIST_UPDATED: u32 = 100;
pub const UPDATE_LEDS: u32 = 1050;
pub const UPDATE_MODE: u32 = 1101;

fn truncated() -> DriverError {
    DriverError::protocol("truncated SDK packet")
}

#[derive(Clone, Debug)]
pub struct Mode {
    pub name: String,
    pub flags: u32,
    pub(super) wire: Vec<u8>,
    pub(super) color_mode_at: usize,
    fixed_prefix: usize,
}

impl Mode {
    /// The server-defined part of the mode, which must match before a backup is replayed.
    pub fn same_definition(&self, other: &Mode) -> bool {
        self.fixed_prefix == other.fixed_prefix
            && self.wire[..self.fixed_prefix] == other.wire[..other.fixed_prefix]
    }
}

#[derive(Clone, Debug)]
pub struct SdkDevice {
    pub index: u32,
    pub identity: String,
    pub name: String,
    pub vendor: String,
    pub kind: u32,
    pub led_count: usize,
    pub active: u32,
    pub modes: Vec<Mode>,
    pub colors: Vec<[u8; 4]>,
}

impl SdkDevice {
    pub fn direct(&self) -> Option<usize> {
        self.modes.iter().position(|m| {
            m.name.eq_ignore_ascii_case("direct")
                && m.flags & DIRECT_FLAGS != 0
                && m.flags & AUTO_SAVE == 0
        })
    }

    /// Direct mode without automatic saving, and one color per LED.
    pub fn writable(&self) -> bool {
        self.led_count > 0 && self.colors.len() == self.led_count && self.direct().is_some()
    }

    /// `device_type` from the SDK documentation.
    pub fn device_kind(&self) -> DeviceKind {
        match self.kind {
            5 | 18 => DeviceKind::Keyboard,
            6 | 7 => DeviceKind::Mouse,
            8 | 9 | 16 => DeviceKind::Headset,
            10 => DeviceKind::Gamepad,
            4 | 11 => DeviceKind::Lighting,
            0..=3 | 14 | 15 => DeviceKind::Component,
            _ => DeviceKind::Other,
        }
    }
}

pub struct Cursor<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, at: 0 }
    }

    pub fn at_end(&self) -> bool {
        self.at == self.data.len()
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], DriverError> {
        let end = self.at.checked_add(n).ok_or_else(truncated)?;
        let bytes = self.data.get(self.at..end).ok_or_else(truncated)?;
        self.at = end;
        Ok(bytes)
    }

    fn u16(&mut self) -> Result<u16, DriverError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub fn u32(&mut self) -> Result<u32, DriverError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn string(&mut self) -> Result<String, DriverError> {
        let n = self.u16()? as usize;
        if !(1..=4096).contains(&n) {
            return Err(DriverError::protocol("SDK string length"));
        }
        let bytes = self.take(n)?;
        if bytes[n - 1] != 0 {
            return Err(DriverError::protocol("SDK string without NUL"));
        }
        Ok(String::from_utf8_lossy(&bytes[..n - 1]).into_owned())
    }

    fn count(&mut self, max: usize) -> Result<usize, DriverError> {
        let n = self.u16()? as usize;
        if n > max {
            return Err(DriverError::protocol("SDK list too long"));
        }
        Ok(n)
    }
}

pub fn parse_mode(c: &mut Cursor<'_>, version: u32) -> Result<Mode, DriverError> {
    let start = c.at;
    let name = c.string()?;
    c.u32()?;
    let flags = c.u32()?;
    let low = c.u32()?;
    let high = c.u32()?;
    let (b_low, b_high) = if version >= 3 {
        (c.u32()?, c.u32()?)
    } else {
        (0, 0)
    };
    let min_colors = c.u32()?;
    let max_colors = c.u32()?;
    let fixed_prefix = c.at - start;
    let speed = c.u32()?;
    let brightness = if version >= 3 { c.u32()? } else { 0 };
    let direction = c.u32()?;
    let color_mode_at = c.at - start;
    let color_mode = c.u32()?;
    let n = c.count(4096)?;
    c.take(n * 4)?;
    let valid = (flags & 1 == 0 || (low <= speed && speed <= high))
        && (version < 3 || flags & 16 == 0 || (b_low <= brightness && brightness <= b_high))
        && direction <= 9
        && color_mode <= 3
        && min_colors <= max_colors
        && (n as u32 <= max_colors || max_colors == 0 && n == 0);
    if !valid {
        return Err(DriverError::protocol("SDK mode out of range"));
    }
    Ok(Mode {
        name,
        flags,
        wire: c.data[start..c.at].to_vec(),
        color_mode_at,
        fixed_prefix,
    })
}

fn skip_zones(c: &mut Cursor<'_>) -> Result<(), DriverError> {
    for _ in 0..c.count(1024)? {
        c.string()?;
        c.take(16)?;
        let matrix = c.u16()? as usize;
        if matrix == 0 {
            continue;
        }
        if matrix < 8 || !(matrix - 8).is_multiple_of(4) {
            return Err(DriverError::protocol("SDK matrix size"));
        }
        let bytes = c.take(matrix)?;
        let h = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let w = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
        let expected = h
            .checked_mul(w)
            .and_then(|n| n.checked_mul(4))
            .and_then(|n| n.checked_add(8));
        if expected != Some(matrix) {
            return Err(DriverError::protocol("SDK matrix dimensions"));
        }
    }
    Ok(())
}

pub fn parse_device(data: &[u8], version: u32, index: u32) -> Result<SdkDevice, DriverError> {
    if !(1..=3).contains(&version) || data.len() > LIMIT {
        return Err(DriverError::protocol("SDK version or size not supported"));
    }
    let mut c = Cursor::new(data);
    if c.u32()? as usize != data.len() {
        return Err(DriverError::protocol("SDK description size"));
    }
    let kind = c.u32()?;
    let name = c.string()?;
    let vendor = c.string()?;
    let description = c.string()?;
    let firmware = c.string()?;
    let serial = c.string()?;
    let location = c.string()?;
    let mode_count = c.count(256)?;
    let active = c.u32()?;
    if mode_count != 0 && active as usize >= mode_count {
        return Err(DriverError::protocol("SDK active mode"));
    }
    let modes = (0..mode_count)
        .map(|_| parse_mode(&mut c, version))
        .collect::<Result<Vec<_>, _>>()?;
    skip_zones(&mut c)?;
    let led_count = c.count(8192)?;
    let mut led_names = Vec::with_capacity(led_count);
    for _ in 0..led_count {
        led_names.push(c.string()?);
        c.u32()?;
    }
    let n = c.count(8192)?;
    let colors = c.take(n * 4)?.as_chunks::<4>().0.to_vec();
    if !c.at_end() {
        return Err(DriverError::protocol("unknown fields in SDK description"));
    }
    // Private identifiers are hashed: neither the serial nor the USB location leaves this function.
    let fingerprint = serde_json::to_vec(&(
        kind,
        &name,
        &vendor,
        &description,
        &firmware,
        &serial,
        &location,
        &led_names,
        modes.iter().map(|m| (&m.name, m.flags)).collect::<Vec<_>>(),
    ))
    .map_err(|e| DriverError::Io(e.to_string()))?;
    Ok(SdkDevice {
        index,
        identity: sha256_hex(&fingerprint),
        name,
        vendor,
        kind,
        led_count,
        active,
        modes,
        colors,
    })
}

/// `ORGB` header plus payload.
pub fn frame(device: u32, packet: u32, payload: &[u8]) -> Result<Vec<u8>, DriverError> {
    if payload.len() > LIMIT {
        return Err(DriverError::protocol("SDK packet too large"));
    }
    let mut wire = Vec::with_capacity(16 + payload.len());
    wire.extend_from_slice(b"ORGB");
    for n in [device, packet, payload.len() as u32] {
        wire.extend_from_slice(&n.to_le_bytes());
    }
    wire.extend_from_slice(payload);
    Ok(wire)
}

/// (device, packet, payload length) of a received header.
pub fn header(h: &[u8; 16]) -> Result<(u32, u32, usize), DriverError> {
    if &h[..4] != b"ORGB" {
        return Err(DriverError::protocol("SDK signature"));
    }
    let mut c = Cursor::new(&h[4..]);
    let (device, packet, n) = (c.u32()?, c.u32()?, c.u32()? as usize);
    if n > LIMIT {
        return Err(DriverError::protocol("SDK packet exceeds limit"));
    }
    Ok((device, packet, n))
}

pub fn mode_payload(mode_index: u32, wire: &[u8]) -> Vec<u8> {
    let mut p = Vec::with_capacity(8 + wire.len());
    p.extend_from_slice(&((8 + wire.len()) as u32).to_le_bytes());
    p.extend_from_slice(&mode_index.to_le_bytes());
    p.extend_from_slice(wire);
    p
}

pub fn colors_payload(colors: &[[u8; 4]]) -> Result<Vec<u8>, DriverError> {
    if colors.len() > 8192 {
        return Err(DriverError::protocol("too many colors"));
    }
    let mut p = Vec::with_capacity(6 + 4 * colors.len());
    p.extend_from_slice(&((6 + 4 * colors.len()) as u32).to_le_bytes());
    p.extend_from_slice(&(colors.len() as u16).to_le_bytes());
    for color in colors {
        p.extend_from_slice(color);
    }
    Ok(p)
}
