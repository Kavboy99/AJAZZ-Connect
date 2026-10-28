use sha2::{Digest, Sha256};

/// Lowercase hex SHA256. Used to hash private identifiers (device paths, serials) before they are
/// stored anywhere, and to fingerprint files.
pub fn sha256_hex(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}
