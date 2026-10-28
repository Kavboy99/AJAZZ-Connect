use super::battery::Reading;
use crate::core::DriverError;

pub(super) const DESCRIPTOR: &[u8] = &[
    6, 2, 255, 9, 2, 161, 1, 133, 19, 9, 2, 21, 0, 38, 255, 0, 117, 8, 149, 19, 129, 0, 9, 2, 21,
    0, 38, 255, 0, 117, 8, 149, 19, 145, 0, 192,
];

fn checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte))
}

/// The only output packet this adapter can construct: a payload-free read, never a settings write.
pub(super) fn request() -> [u8; 20] {
    let mut packet = [0; 20];
    packet[..5].copy_from_slice(&[0x13, 0x4a, 1, 0, 0]);
    packet[19] = checksum(&packet[..19]);
    packet
}

pub(super) fn reply(packet: &[u8]) -> Result<Reading, DriverError> {
    if packet.len() != 20
        || packet[..5] != [0x13, 0x4a, 1, 0, 2]
        || checksum(&packet[..19]) != packet[19]
    {
        return Err(DriverError::protocol(
            "invalid radio battery frame or checksum",
        ));
    }
    Reading::parse(&packet[5..7])
}
