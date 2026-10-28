//! Pure packets of the classic SONiX Feature protocol. Normal lighting settings only: no firmware
//! download, per-key table, macros, TFT or streaming commands.
//! Source: official-driver research in OpenKey (`research/SOURCE-MANIFEST.json`).

use crate::core::DriverError;

pub const REPORT_BYTES: usize = 65;
pub const START: u8 = 0x18;
pub const SELECT: u8 = 0x13;
pub const SAVE: u8 = 0x02;
pub const FINISH: u8 = 0xf0;

pub fn command(opcode: u8, argument: u8) -> Result<[u8; REPORT_BYTES], DriverError> {
    if !matches!(opcode, START | SELECT | SAVE | FINISH) {
        return Err(DriverError::rejected(format!(
            "opcode {opcode:02X} is not allowed"
        )));
    }
    if !(argument == 0 || opcode == SELECT && argument == 1) {
        return Err(DriverError::rejected("argument is not allowed"));
    }
    let mut p = [0; REPORT_BYTES];
    p[1] = 4;
    p[2] = opcode;
    p[9] = argument;
    Ok(p)
}

/// The exact acknowledgement of `opcode`; a stale or foreign reply is refused.
pub fn ack(packet: &[u8], opcode: u8) -> Result<(), DriverError> {
    let exact = packet.len() == REPORT_BYTES
        && packet[0] == 0
        && packet[1] == 4
        && packet[2] == opcode
        && packet[4] == 1;
    if exact {
        Ok(())
    } else {
        Err(DriverError::Unconfirmed(format!(
            "no exact ACK for {opcode:02X}"
        )))
    }
}

pub fn solid(rgb: [u8; 3]) -> [u8; REPORT_BYTES] {
    let mut p = [0; REPORT_BYTES];
    p[1] = 1;
    p[2..5].copy_from_slice(&rgb);
    p[10] = 5;
    p[11] = 3;
    p[15] = 0xaa;
    p[16] = 0x55;
    p
}

/// START, SELECT, data, SAVE, then FINISH even after a failure. SAVE is never sent when the data
/// transfer failed, and nothing is retried.
pub fn apply_transaction(
    mut send: impl FnMut(&[u8; REPORT_BYTES], bool) -> Result<(), DriverError>,
    rgb: [u8; 3],
) -> Result<(), DriverError> {
    send(&command(START, 0)?, true)?;
    let result = (|| {
        send(&command(SELECT, 1)?, true)?;
        send(&solid(rgb), false)?;
        send(&command(SAVE, 0)?, true)
    })();
    let finish = send(&command(FINISH, 0)?, false);
    result?;
    finish
}
