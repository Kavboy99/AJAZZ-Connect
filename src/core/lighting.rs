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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightingSettings {
    pub preset: u8,
    pub brightness: u8,
    pub speed: u8,
    /// None retains the device's saved color slot and palette.
    pub color: Option<Rgb>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LightingPreset {
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
