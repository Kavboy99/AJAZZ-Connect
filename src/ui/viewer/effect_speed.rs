//! Preview timing never treats the lowest firmware speed as an off switch.
pub(crate) fn factor(speed: u8) -> f32 {
    (f32::from(speed) * 0.5).max(0.1)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zero_moves_slowly_and_higher_settings_are_faster() {
        assert!(factor(0) > 0.0);
        assert!(factor(0) < factor(1) / 4.0);
        for speed in 1..4 {
            assert!(factor(speed) < factor(speed + 1));
        }
    }
}
