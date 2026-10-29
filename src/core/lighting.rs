//! Driver-owned lighting programs. Numeric ids are opaque to the application and UI.
use super::Rgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightingEffect {
    Solid,
    Rainbow,
    Breathing,
    Flash,
    Rain,
    RainbowWheel,
    Ripples,
    Stars,
    Fade,
    Snake,
    Neon,
    Reactive,
    Sine,
    Scan,
    Windmill,
    Waterfall,
    Blossom,
    Storm,
    Off,
}

impl LightingEffect {
    pub fn key_triggered(self) -> bool {
        matches!(self, Self::Flash | Self::Ripples | Self::Reactive)
    }
    pub fn continuous(self) -> bool {
        !self.key_triggered()
            && !matches!(
                self,
                Self::Solid | Self::Off | Self::Fade | Self::Scan | Self::Storm
            )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LightingSettings {
    pub preset: u8,
    pub brightness: u8,
    pub speed: u8,
    /// None retains the device's saved color slot and palette.
    pub color: Option<Rgb>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LightingPreset {
    pub device_color: Option<Rgb>,
    pub effect: LightingEffect,
    pub saved: LightingSettings,
    pub brightness_max: Option<u8>,
    pub speed_max: Option<u8>,
    pub custom_color: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LightingEditor {
    pub presets: Vec<LightingPreset>,
    pub current: u8,
}

/// Static controls for a driver preview; values are illustrative, never a device read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LightingPreview {
    pub editor: LightingEditor,
    pub requires_usb: bool,
}
