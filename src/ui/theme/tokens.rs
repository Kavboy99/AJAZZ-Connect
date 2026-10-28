//! Design tokens from `docs/DESIGN.md`. The only place with color and size literals; widget code
//! refers to these names. When a color changes, recompute the contrast table in the document.

use eframe::egui::Color32;

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// A color token: its dark and light values. Widgets read it with `get()`, which follows the
/// current theme and mixes the two while the theme is changing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    dark: Color32,
    light: Color32,
}

impl Token {
    const fn new(dark: u32, light: u32) -> Self {
        Self {
            dark: hex(dark),
            light: hex(light),
        }
    }

    pub fn get(self) -> Color32 {
        let t = super::mode::blend();
        if t <= 0.0 {
            self.dark
        } else if t >= 1.0 {
            self.light
        } else {
            self.dark.lerp_to_gamma(self.light, t)
        }
    }

    #[cfg(test)]
    pub const fn dark(self) -> Color32 {
        self.dark
    }

    #[cfg(test)]
    pub const fn light(self) -> Color32 {
        self.light
    }
}

/// Dark values are the primary theme; light values follow the WinUI 3 light tokens.
pub mod color {
    use super::Token;

    pub const SURFACE_SUNKEN: Token = Token::new(0x1A1A1A, 0xEEEEEE);
    pub const SURFACE_BASE: Token = Token::new(0x202020, 0xF3F3F3);
    pub const SURFACE_CARD: Token = Token::new(0x2B2B2B, 0xFBFBFB);
    pub const SURFACE_CARD_HOVER: Token = Token::new(0x303030, 0xF6F6F6);
    pub const SURFACE_OVERLAY: Token = Token::new(0x2C2C2C, 0xF9F9F9);
    pub const CONTROL_FILL: Token = Token::new(0x343434, 0xFDFDFD);
    pub const CONTROL_FILL_HOVER: Token = Token::new(0x3A3A3A, 0xF6F6F6);
    pub const CONTROL_FILL_PRESSED: Token = Token::new(0x2F2F2F, 0xF0F0F0);
    pub const CONTROL_FILL_DISABLED: Token = Token::new(0x2A2A2A, 0xF5F5F5);

    pub const STROKE_SUBTLE: Token = Token::new(0x363636, 0xE5E5E5);
    pub const STROKE_DEFAULT: Token = Token::new(0x404040, 0xD1D1D1);
    pub const STROKE_CONTROL: Token = Token::new(0x7F7F7F, 0x8A8A8A);
    pub const FOCUS_RING: Token = Token::new(0xE6E6E6, 0x1F1F1F);

    pub const TEXT_PRIMARY: Token = Token::new(0xF2F2F2, 0x1B1B1B);
    pub const TEXT_SECONDARY: Token = Token::new(0xC8C8C8, 0x4F4F4F);
    pub const TEXT_TERTIARY: Token = Token::new(0xA0A0A0, 0x6B6B6B);
    pub const TEXT_DISABLED: Token = Token::new(0x6E6E6E, 0xA3A3A3);
    pub const TEXT_ON_ACCENT: Token = Token::new(0x101010, 0xFAFAFA);

    pub const ACCENT: Token = Token::new(0x4DB2E8, 0x005FB8);
    pub const ACCENT_HOVER: Token = Token::new(0x62BDEC, 0x0A6AC4);
    pub const ACCENT_PRESSED: Token = Token::new(0x3F9FD8, 0x004C93);
    pub const ACCENT_SUBTLE: Token = Token::new(0x243F50, 0xD3E5F5);

    pub const STATUS_SUCCESS: Token = Token::new(0x6CCB5F, 0x0F7B0F);
    pub const STATUS_CAUTION: Token = Token::new(0xE0B84A, 0x8F5500);
    pub const STATUS_CRITICAL: Token = Token::new(0xE8808A, 0xC42B1C);
    pub const STATUS_INFO: Token = ACCENT;

    /// The light behind a hovered device model and its rim light (`docs/DESIGN.md`, 8.1).
    pub const DEVICE_GLOW: Token = Token::new(0x4DB2E8, 0x62BDEC);
    /// The soft contact shadow under a device model.
    pub const DEVICE_SHADOW: Token = Token::new(0x0E0E0E, 0x8C8C8C);
}

/// The 4 px grid. 48 joins when a layout needs it.
pub mod space {
    pub const XS: f32 = 4.0;
    pub const S: f32 = 8.0;
    pub const M: f32 = 12.0;
    pub const L: f32 = 16.0;
    pub const XL: f32 = 24.0;
}

pub mod size {
    pub const PAGE_MARGIN: f32 = 24.0;
    pub const CARD_PADDING: f32 = 16.0;
    pub const CARD_GAP: f32 = 12.0;
    pub const CONTROL_HEIGHT: f32 = 32.0;
    pub const NAV_ROW: f32 = 40.0;
    pub const NAV_COLLAPSED: f32 = 48.0;
    pub const NAV_EXPANDED: f32 = 240.0;
    pub const NAV_INDICATOR: f32 = 3.0;
    pub const ICON_SMALL: f32 = 16.0;
    pub const ICON: f32 = 20.0;
    pub const ICON_LARGE: f32 = 24.0;
    pub const STROKE: f32 = 1.0;
    pub const FOCUS_RING: f32 = 2.0;
    pub const FOCUS_OFFSET: f32 = 2.0;
    /// Long lines stay under ~75 characters of body text.
    pub const READABLE_WIDTH: f32 = 640.0;
    /// Device tiles never get narrower; the grid adds a column when two more fit.
    pub const DEVICE_TILE_MIN: f32 = 224.0;
}

/// Adaptive layout (`docs/DESIGN.md`, section 5.1). Window widths follow the WinUI size classes;
/// content widths decide how parts of a page stand next to each other.
pub mod breakpoint {
    /// Window: from this width the navigation pane is expanded (WinUI "expanded" class).
    pub const NAV_EXPANDED: f32 = 1008.0;
    /// Content: below this, the side-by-side parts of one card stack.
    pub const SECTION_COLUMNS: f32 = 560.0;
    /// Content: from this width, the cards of a page stand in two columns.
    pub const PAGE_COLUMNS: f32 = 960.0;
}

pub mod radius {
    pub const CONTROL: u8 = 4;
    pub const OVERLAY: u8 = 8;
}

pub mod motion {
    /// Press feedback scale.
    pub const PRESS_SCALE: f32 = 0.97;
    /// Rise of an entering page.
    pub const PAGE_RISE: f32 = 20.0;
    /// Shift of appearing badges and popups.
    pub const APPEAR_SHIFT: f32 = 8.0;
    /// Scale an entering detail page starts from.
    pub const DRILL_SCALE: f32 = 0.96;
    /// Delay before a tooltip appears, seconds.
    pub const TOOLTIP_DELAY: f32 = 0.5;
    /// How far a device model turns toward the pointer, radians.
    pub const TILT_YAW: f32 = 0.24;
    pub const TILT_PITCH: f32 = 0.12;
    /// How far a hovered device model rises on the tile, px.
    pub const TILE_RISE: f32 = 8.0;
}

#[cfg(test)]
mod tests {
    use super::{Color32, Token, color};

    fn luminance(c: Color32) -> f32 {
        let linear = |v: u8| {
            let s = v as f32 / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(c.r()) + 0.7152 * linear(c.g()) + 0.0722 * linear(c.b())
    }

    fn contrast(a: Color32, b: Color32) -> f32 {
        let (x, y) = (luminance(a), luminance(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    /// The contrast table of `docs/DESIGN.md`, section 3, for both themes.
    #[test]
    fn both_palettes_meet_the_contrast_table() {
        use color::*;
        let text = 4.5;
        let pairs: &[(Token, Token, f32)] = &[
            (TEXT_PRIMARY, SURFACE_BASE, text),
            (TEXT_PRIMARY, SURFACE_CARD, text),
            (TEXT_SECONDARY, SURFACE_BASE, text),
            (TEXT_SECONDARY, SURFACE_CARD, text),
            (TEXT_TERTIARY, SURFACE_BASE, text),
            (TEXT_TERTIARY, SURFACE_CARD, text),
            (TEXT_TERTIARY, CONTROL_FILL, text),
            (TEXT_PRIMARY, SURFACE_SUNKEN, text),
            (ACCENT, ACCENT_SUBTLE, text),
            (TEXT_PRIMARY, ACCENT_SUBTLE, text),
            (FOCUS_RING, SURFACE_CARD, 3.0),
            (TEXT_ON_ACCENT, ACCENT, text),
            (TEXT_ON_ACCENT, ACCENT_HOVER, text),
            (TEXT_ON_ACCENT, ACCENT_PRESSED, text),
            (ACCENT, SURFACE_BASE, text),
            (ACCENT, SURFACE_CARD, text),
            (STATUS_SUCCESS, SURFACE_CARD, text),
            (STATUS_CAUTION, SURFACE_CARD, text),
            (STATUS_CRITICAL, SURFACE_CARD, text),
            (STATUS_SUCCESS, SURFACE_SUNKEN, text),
            (STATUS_CAUTION, SURFACE_SUNKEN, text),
            (STATUS_CRITICAL, SURFACE_SUNKEN, text),
            (STROKE_CONTROL, SURFACE_CARD, 3.0),
        ];
        for (i, (fg, bg, min)) in pairs.iter().enumerate() {
            for (theme, f, b) in [
                ("dark", fg.dark(), bg.dark()),
                ("light", fg.light(), bg.light()),
            ] {
                let ratio = contrast(f, b);
                assert!(ratio >= *min, "pair {i} ({theme}): {ratio:.2} < {min}");
            }
        }
    }

    #[test]
    fn no_pure_white_or_black_in_surfaces_and_strokes() {
        use color::*;
        for token in [
            SURFACE_SUNKEN,
            SURFACE_BASE,
            SURFACE_CARD,
            SURFACE_CARD_HOVER,
            SURFACE_OVERLAY,
            CONTROL_FILL,
            CONTROL_FILL_HOVER,
            CONTROL_FILL_PRESSED,
            CONTROL_FILL_DISABLED,
            STROKE_SUBTLE,
            STROKE_DEFAULT,
            STROKE_CONTROL,
            ACCENT,
            ACCENT_SUBTLE,
        ] {
            for c in [token.dark(), token.light()] {
                assert!(c != Color32::WHITE && c != Color32::BLACK, "{c:?}");
            }
        }
    }
}
