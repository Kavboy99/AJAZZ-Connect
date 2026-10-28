use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InspectError {
    TooLarge,
    /// HTML, HEX, ZIP or another format that is not a raw image or a vendor EXE.
    NotFirmware,
    /// A recognized container with damaged or unexpected structure; English detail.
    Malformed(String),
    Io(String),
}

impl InspectError {
    pub(super) fn malformed(detail: &str) -> Self {
        Self::Malformed(detail.into())
    }
}

impl fmt::Display for InspectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => f.write_str("file exceeds 32 MB"),
            Self::NotFirmware => f.write_str("not a vendor EXE or raw ARM image"),
            Self::Malformed(detail) => write!(f, "malformed package: {detail}"),
            Self::Io(detail) => write!(f, "io: {detail}"),
        }
    }
}

impl std::error::Error for InspectError {}
