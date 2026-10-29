//! HSV operates directly on RGB channel bytes; display gamma must not inflate LED channels.
use eframe::egui::{
    Color32,
    ecolor::{Hsva, linear_u8_from_linear_f32},
};

pub(super) fn color(hue: f32, saturation: f32) -> Color32 {
    let [r, g, b] = Hsva::new(hue, saturation, 1.0, 1.0)
        .to_rgb()
        .map(linear_u8_from_linear_f32);
    Color32::from_rgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn half_saturation_keeps_half_channel_values_instead_of_gamma_whitening() {
        assert_eq!(color(0.5, 0.5), Color32::from_rgb(128, 255, 255));
        assert_eq!(color(0.5, 1.0), Color32::from_rgb(0, 255, 255));
        assert_eq!(color(0.5, 0.0), Color32::WHITE);
    }
}
