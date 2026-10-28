use serde::{Deserialize, Serialize};

/// A color chosen by the user for a device. It is data, so it is shown exactly as picked.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn to_array(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    /// `#RRGGBB`, the form users type and read.
    pub fn hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Accepts `RRGGBB` with or without a leading `#`.
    pub fn parse_hex(text: &str) -> Option<Self> {
        let digits = text.trim().trim_start_matches('#');
        if digits.len() != 6 || !digits.is_ascii() {
            return None;
        }
        let channel = |at: usize| u8::from_str_radix(&digits[at..at + 2], 16).ok();
        Some(Self::new(channel(0)?, channel(2)?, channel(4)?))
    }
}

#[cfg(test)]
mod tests {
    use super::Rgb;

    #[test]
    fn hex_round_trip_and_rejects_garbage() {
        let color = Rgb::new(0x4d, 0xb2, 0xe8);
        assert_eq!(color.hex(), "#4DB2E8");
        assert_eq!(Rgb::parse_hex("#4db2e8"), Some(color));
        assert_eq!(Rgb::parse_hex(" 4DB2E8 "), Some(color));
        for bad in ["", "#12345", "#1234567", "#GG0000", "#ЖЖ0000"] {
            assert_eq!(Rgb::parse_hex(bad), None, "{bad}");
        }
    }
}
