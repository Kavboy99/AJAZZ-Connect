//! Finite light persistence in model space, so camera movement never smears the device.
use crate::core::LightingEffect;

pub(crate) const LIFETIME: f32 = 2.0;

pub(super) fn strength(effect: LightingEffect, index: usize, source: usize, age: f32) -> f32 {
    if !(0.0..LIFETIME).contains(&age) {
        return 0.0;
    }
    let dx = (index % 14) as f32 / 13.0 - (source % 14) as f32 / 13.0;
    let dy = ((index / 14) as f32 - (source / 14) as f32) / 13.0;
    let distance = match effect {
        LightingEffect::Ripples => dx.hypot(dy),
        LightingEffect::Flash if index / 14 == source / 14 => dx.abs(),
        LightingEffect::Flash => return 0.0,
        _ if index == source => {
            return (-age
                * if effect == LightingEffect::Fade {
                    2.5
                } else {
                    5.0
                })
            .exp()
                * ending(age);
        }
        _ => return 0.0,
    };
    let behind = age * 0.8 - distance;
    // A short leading edge and a longer, exponential wake preserve the moving bright crest.
    let wave = if behind < 0.0 {
        (1.0 + behind / 0.06).clamp(0.0, 1.0).powi(2)
    } else {
        (-behind / 0.18).exp()
    };
    wave * ending(age)
}

fn ending(age: f32) -> f32 {
    let t = ((LIFETIME - age) / 0.35).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flash_has_a_bright_crest_and_a_local_decaying_wake() {
        let source = 14;
        let index = source + 5;
        let arrival = (5.0 / 13.0) / 0.8;
        assert!(strength(LightingEffect::Flash, index, source, arrival) > 0.99);
        let tail = strength(LightingEffect::Flash, index, source, arrival + 0.2);
        assert!(tail > 0.35 && tail < 0.5);
        assert!(strength(LightingEffect::Flash, index, source, arrival + 0.4) < tail);
        assert_eq!(
            strength(LightingEffect::Flash, index + 14, source, arrival),
            0.0
        );
        assert_eq!(
            strength(LightingEffect::Flash, index, source, arrival - 0.2),
            0.0
        );
    }

    #[test]
    fn every_reactive_tail_expires_without_a_frame_history() {
        for effect in [
            LightingEffect::Flash,
            LightingEffect::Ripples,
            LightingEffect::Fade,
            LightingEffect::Reactive,
        ] {
            for index in 0..84 {
                assert_eq!(strength(effect, index, 32, LIFETIME), 0.0);
                assert!(strength(effect, index, 32, LIFETIME - 0.001) < 0.0001);
            }
        }
        assert!(strength(LightingEffect::Ripples, 32, 32, 0.2) > 0.35);
        assert_eq!(strength(LightingEffect::Reactive, 33, 32, 0.0), 0.0);
    }
}
