use std::fmt;

/// Typed driver failure. User-facing text is built by the interface from the variant; the
/// `String` payloads are English technical details for logs and diagnostic reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DriverError {
    /// A service or OS facility the driver needs is not reachable (a local server is off, the
    /// platform has no HID access).
    Unavailable(String),
    /// Another program holds the device.
    Busy,
    /// The device does not match the protocol exactly; nothing was written.
    Rejected(String),
    /// A reply did not match the request.
    Protocol(String),
    /// A write was not acknowledged. It is never retried: the packet may have arrived.
    Unconfirmed(String),
    /// The write was acknowledged but reading the state back gave a different value.
    ReadBackMismatch,
    /// The device, its firmware or its position changed since discovery.
    Changed,
    /// There is nothing to roll back to.
    NoRollback,
    /// The backup belongs to another device or is malformed.
    ForeignBackup,
    /// Transport or file failure.
    Io(String),
}

impl DriverError {
    pub fn rejected(detail: impl Into<String>) -> Self {
        Self::Rejected(detail.into())
    }

    pub fn protocol(detail: impl Into<String>) -> Self {
        Self::Protocol(detail.into())
    }
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(detail) => write!(f, "unavailable: {detail}"),
            Self::Busy => f.write_str("device is held by another program"),
            Self::Rejected(detail) => write!(f, "device rejected: {detail}"),
            Self::Protocol(detail) => write!(f, "protocol mismatch: {detail}"),
            Self::Unconfirmed(detail) => write!(f, "write not confirmed, not retried: {detail}"),
            Self::ReadBackMismatch => f.write_str("read-back differs from the written state"),
            Self::Changed => f.write_str("device changed since discovery"),
            Self::NoRollback => f.write_str("nothing to roll back"),
            Self::ForeignBackup => f.write_str("backup belongs to another device"),
            Self::Io(detail) => write!(f, "io: {detail}"),
        }
    }
}

impl std::error::Error for DriverError {}

impl From<std::io::Error> for DriverError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}
