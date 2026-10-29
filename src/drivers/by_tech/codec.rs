//! Pure framing of the BY Tech (BYCOMBO4) configuration channel: Feature report 6, 520 bytes.
//! Source: static analysis of `OemDrv.exe` from `AJAZZ_AK820_MAX_driver_V1.0` (class `CDevG5KB`,
//! `docs/DEVICES.md`, section 1.1). Only the commands below exist here; macros, key matrix, game
//! mode, screen, music and anything firmware-related are deliberately absent.

use crate::core::DriverError;

pub const REPORT_ID: u8 = 6;
/// Report ID plus 519 bytes, as the descriptor declares (`96 07 02`).
pub const REPORT_BYTES: usize = 520;
const HEADER_BYTES: usize = 8;
const DATA_BYTES: usize = REPORT_BYTES - HEADER_BYTES;

pub const PASSWORD_BYTES: usize = 6;
/// The six bytes the vendor software compares with `Psd` in `KB.ini` before it talks to the
/// keyboard: the single-mode board (`Dev/kb/Wired`) and the tri-mode board (`Dev/kb/Thr`). Both
/// `KB.ini` files are identical in everything this driver touches (`Fw`, `CRC`, `LedOpt`,
/// `RGBIndex`). The tri-mode answer was read from the owner's keyboard over USB on 2026-10-28.
pub const PASSWORDS: [[u8; PASSWORD_BYTES]; 2] = [
    [0x03, 0x00, 0x00, 0x00, 0x01, 0x42],
    [0x03, 0x00, 0x00, 0x00, 0x01, 0x5e],
];

/// Every request this driver can build. Reads have the high bit set; a write is `read & 0x7F`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    ReadPassword,
    ReadPower,
    ReadConfig,
    WriteConfig,
    ReadColors,
    WriteColors,
}

impl Command {
    fn opcode(self) -> u8 {
        match self {
            Self::ReadPassword => 0x82,
            Self::ReadPower => 0x87,
            Self::ReadConfig => 0x84,
            Self::WriteConfig => 0x04,
            Self::ReadColors => 0x8a,
            Self::WriteColors => 0x0a,
        }
    }

    /// The vendor sends sub-index 1 only with the password request.
    fn sub_index(self) -> u8 {
        match self {
            Self::ReadPassword => 1,
            Self::ReadPower => 2,
            _ => 0,
        }
    }

    /// Fixed transfer length; anything else is refused.
    pub fn length(self) -> usize {
        match self {
            Self::ReadPassword => PASSWORD_BYTES,
            Self::ReadPower => 2,
            Self::ReadConfig | Self::WriteConfig => super::state::CONFIG_BYTES,
            Self::ReadColors | Self::WriteColors => super::state::COLOR_TABLE_BYTES,
        }
    }

    pub fn is_write(self) -> bool {
        self.opcode() & 0x80 == 0
    }
}

/// `[id, opcode, sub, 0, pages, page, len_lo, len_hi]`. Every transfer here fits one page.
fn header(command: Command) -> [u8; REPORT_BYTES] {
    let length = command.length();
    let mut p = [0; REPORT_BYTES];
    p[0] = REPORT_ID;
    p[1] = command.opcode();
    p[2] = command.sub_index();
    p[4] = 1;
    p[6..8].copy_from_slice(&(length as u16).to_le_bytes());
    p
}

/// A read request: the device prepares the answer for the following Get Feature.
pub fn request(command: Command) -> Result<[u8; REPORT_BYTES], DriverError> {
    if command.is_write() {
        return Err(DriverError::rejected("a write needs data"));
    }
    Ok(header(command))
}

/// A write of exactly `command.length()` bytes.
pub fn write(command: Command, data: &[u8]) -> Result<[u8; REPORT_BYTES], DriverError> {
    if !command.is_write() || data.len() != command.length() || data.len() > DATA_BYTES {
        return Err(DriverError::rejected(
            "write size does not match the command",
        ));
    }
    let mut p = header(command);
    p[HEADER_BYTES..HEADER_BYTES + data.len()].copy_from_slice(data);
    Ok(p)
}

/// The data of a Get Feature answer. Windows reports only the bytes the keyboard sent (header plus
/// data), and the keyboard echoes the request header, so the frame must carry the Report ID, the
/// same command and at least the requested data; the content is checked by its own validators.
pub fn reply(report: &[u8], command: Command) -> Result<&[u8], DriverError> {
    let framed = report.len() >= HEADER_BYTES + command.length()
        && report.len() <= REPORT_BYTES
        && report[0] == REPORT_ID
        && report[1] == command.opcode();
    let framed = framed && report[..HEADER_BYTES] == header(command)[..HEADER_BYTES];
    if !framed {
        // The size and the first bytes go to the diagnostic report: they show what the frame was.
        return Err(DriverError::protocol(format!(
            "truncated or foreign Feature report: {} bytes, starts {}",
            report.len(),
            hex(&report[..report.len().min(16)])
        )));
    }
    Ok(&report[HEADER_BYTES..HEADER_BYTES + command.length()])
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
