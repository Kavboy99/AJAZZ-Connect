use crate::core::Rgb;
use eframe::egui::Color32;

/// Hue, saturation and value in 0..1. Kept by the picker so the hue survives grey colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl Hsv {
    pub fn to_rgb(self) -> Rgb {
        let h = (self.h.rem_euclid(1.0)) * 6.0;
        let c = self.v * self.s;
        let x = c * (1.0 - (h % 2.0 - 1.0).abs());
        let (r, g, b) = match h as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let m = self.v - c;
        let byte = |f: f32| ((f + m).clamp(0.0, 1.0) * 255.0).round() as u8;
        Rgb::new(byte(r), byte(g), byte(b))
    }

    /// Keeps `hue` when the color has none (greys), so the plane does not jump to red.
    pub fn from_rgb(rgb: Rgb, hue: f32) -> Self {
        let [r, g, b] = rgb.to_array().map(|c| c as f32 / 255.0);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let d = max - min;
        let h = if d == 0.0 {
            hue
        } else if max == r {
            ((g - b) / d).rem_euclid(6.0) / 6.0
        } else if max == g {
            ((b - r) / d + 2.0) / 6.0
        } else {
            ((r - g) / d + 4.0) / 6.0
        };
        Self {
            h,
            s: if max == 0.0 { 0.0 } else { d / max },
            v: max,
        }
    }

    pub fn color32(self) -> Color32 {
        let c = self.to_rgb();
        Color32::from_rgb(c.r, c.g, c.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_keeps_hue_for_greys() {
        for rgb in [
            Rgb::new(77, 178, 232),
            Rgb::new(255, 0, 0),
            Rgb::new(10, 200, 30),
        ] {
            assert_eq!(Hsv::from_rgb(rgb, 0.0).to_rgb(), rgb);
        }
        assert_eq!(Hsv::from_rgb(Rgb::new(128, 128, 128), 0.4).h, 0.4);
    }
}
