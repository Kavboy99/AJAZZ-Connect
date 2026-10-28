//! The two lighting blocks the keyboard keeps in its memory, as `OemDrv.exe` builds them
//! (`CDevG5KB`, `docs/DEVICES.md`, section 1.1). Only the selected effect's fields are changed;
//! everything else is written back exactly as it was read.

use crate::core::DriverError;

pub const CONFIG_BYTES: usize = 128;
pub const COLOR_TABLE_BYTES: usize = 512;

/// Index of the "game mode" lighting flag; the vendor clears it when it selects an effect.
const GAME_MODE: usize = 0x09;
const EFFECT: usize = 0x0a;
/// Hardware effect ids in `KB.ini` (`LedOpt*`, first column) go up to 21.
const LAST_EFFECT: u8 = 21;
/// `LedOpt1 = 1,...`: "Fixed_on", one color for every key.
const STATIC_EFFECT: u8 = 1;
/// Per-effect settings start at 0x38: `[speed, brightness << 4 | color slot]` per effect id.
const SETTINGS: usize = 0x38;
/// `LightHW = 0,1,2,3,4`.
const MAX_BRIGHTNESS: u8 = 4;
/// `RandomVal` in `KB.ini` defaults to 7: the slot that means "random colors".
const RANDOM_SLOT: u8 = 7;
const STATIC_SLOT: u8 = 0;

/// Seven RGB slots per effect id; id 0 is unused. `RGBIndex = 0,1,2` keeps the R, G, B order.
const ENTRY_BYTES: usize = 21;
/// With `CRC = 1` the vendor pads the table to 512 bytes and marks it with `5A A5` at 0x1FA.
const SIGNATURE_AT: usize = 0x1fa;
const SIGNATURE: [u8; 2] = [0x5a, 0xa5];

/// The 128-byte configuration block (commands 0x84 / 0x04).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config([u8; CONFIG_BYTES]);

impl Config {
    /// Refuses a block whose lighting fields are outside what the vendor software ever writes.
    pub fn parse(data: &[u8]) -> Result<Self, DriverError> {
        let block: [u8; CONFIG_BYTES] = data
            .try_into()
            .map_err(|_| DriverError::protocol("configuration block size"))?;
        let config = Self(block);
        let (brightness, slot) = config.static_settings();
        if config.effect() > LAST_EFFECT || brightness > MAX_BRIGHTNESS || slot > RANDOM_SLOT {
            return Err(DriverError::protocol("unknown lighting configuration"));
        }
        for effect in 1..=18 {
            let (speed, brightness, slot) = config.settings(effect);
            if speed > 4 || brightness > MAX_BRIGHTNESS || slot > RANDOM_SLOT {
                return Err(DriverError::protocol("unknown effect settings"));
            }
        }
        Ok(config)
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn effect(&self) -> u8 {
        self.0[EFFECT]
    }

    fn static_settings(&self) -> (u8, u8) {
        let packed = self.0[SETTINGS + 2 * usize::from(STATIC_EFFECT) + 1];
        (packed >> 4, packed & 0x0f)
    }

    pub(super) fn settings(&self, effect: u8) -> (u8, u8, u8) {
        let at = SETTINGS + 2 * usize::from(effect);
        (self.0[at], self.0[at + 1] >> 4, self.0[at + 1] & 0x0f)
    }

    pub(super) fn with_program(
        &self,
        s: crate::core::LightingSettings,
        brightness: bool,
        speed: bool,
    ) -> Self {
        let mut block = self.0;
        block[GAME_MODE] = 0;
        block[EFFECT] = s.preset;
        let at = SETTINGS + 2 * usize::from(s.preset);
        if speed {
            block[at] = s.speed;
        }
        if brightness {
            block[at + 1] = s.brightness << 4 | (block[at + 1] & 0x0f);
        }
        if s.color.is_some() {
            block[at + 1] &= 0xf0;
        }
        Self(block)
    }

    /// The static effect in slot 0. A brightness of zero would hide the color, so it becomes the
    /// maximum; any other brightness and the effect speed stay as they were.
    pub fn solid(&self) -> Self {
        let mut block = self.0;
        let (brightness, _) = self.static_settings();
        let brightness = if brightness == 0 {
            MAX_BRIGHTNESS
        } else {
            brightness
        };
        block[GAME_MODE] = 0;
        block[EFFECT] = STATIC_EFFECT;
        block[SETTINGS + 2 * usize::from(STATIC_EFFECT) + 1] = brightness << 4 | STATIC_SLOT;
        Self(block)
    }
}

/// The 512-byte color table (commands 0x8A / 0x0A).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorTable([u8; COLOR_TABLE_BYTES]);

impl ColorTable {
    pub fn parse(data: &[u8]) -> Result<Self, DriverError> {
        let table: [u8; COLOR_TABLE_BYTES] = data
            .try_into()
            .map_err(|_| DriverError::protocol("color table size"))?;
        if table[SIGNATURE_AT..SIGNATURE_AT + 2] != SIGNATURE {
            return Err(DriverError::protocol("color table without signature"));
        }
        Ok(Self(table))
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn static_color(&self) -> [u8; 3] {
        let at = Self::static_slot();
        [self.0[at], self.0[at + 1], self.0[at + 2]]
    }

    pub fn with_static_color(&self, rgb: [u8; 3]) -> Self {
        let mut table = self.0;
        let at = Self::static_slot();
        table[at..at + 3].copy_from_slice(&rgb);
        Self(table)
    }

    pub(super) fn with_effect_color(&self, effect: u8, rgb: [u8; 3]) -> Self {
        let mut table = self.0;
        let at = ENTRY_BYTES * usize::from(effect);
        table[at..at + 3].copy_from_slice(&rgb);
        Self(table)
    }

    fn static_slot() -> usize {
        ENTRY_BYTES * usize::from(STATIC_EFFECT) + 3 * usize::from(STATIC_SLOT)
    }
}
