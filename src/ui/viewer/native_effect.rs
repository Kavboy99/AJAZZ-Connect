//! Illustrative spatial models of the named firmware effects, not captured firmware timing.
use crate::core::LightingEffect as E;
use eframe::egui::{Color32, ecolor::Hsva};
use std::f32::consts::TAU;

pub(crate) fn sample(effect: E, x: f32, y: f32, time: f32, press: f32, chosen: Color32) -> Color32 {
    let time = if matches!(effect, E::RainbowWheel | E::Windmill | E::Blossom) {
        time
    } else {
        time / 5.0
    };
    let dx = x - 0.5;
    let dy = (y - 0.5) * 0.4;
    let radius = dx.hypot(dy);
    let angle = dy.atan2(dx) / TAU;
    let pulse = |v: f32| (0.5 + 0.5 * (v * TAU).cos()).powi(8);
    let rainbow = |h: f32| Color32::from(Hsva::new(h.rem_euclid(1.0), 1.0, 1.0, 1.0));
    let hash = (x * 127.1 + y * 311.7).sin() * 43_758.547;
    let seed = hash.fract().abs();
    let (color, strength) = match effect {
        E::Solid => (chosen, 1.0),
        E::Breathing => (chosen, 0.12 + 0.88 * (time * TAU * 0.5).sin().powi(2)),
        E::Rainbow => (rainbow(time * 0.3), 1.0),
        E::Flash => (chosen, press * pulse(x - time * 0.9)),
        E::Rain => (chosen, pulse(y - time * (0.5 + seed)) * seed),
        E::RainbowWheel => (rainbow(angle + time * 0.4), 1.0),
        E::Ripples => (chosen, press * pulse(radius * 4.0 - time)),
        E::Stars => (chosen, pulse(seed * 3.0 + time * (0.2 + seed))),
        E::Fade => (chosen, 0.0),
        E::Snake => {
            let path = (y * 4.0).floor()
                + if (y * 4.0).floor() as i32 % 2 == 0 {
                    x
                } else {
                    1.0 - x
                };
            (chosen, (1.0 - (path - time * 2.0).rem_euclid(4.0)).max(0.0))
        }
        E::Neon => (rainbow(time * 0.25 + x * 0.08), 1.0 - pulse(x - time)),
        E::Reactive => (chosen, press),
        E::Sine => (
            chosen,
            pulse(y - 0.5 - (x * TAU).sin() * 0.2 - (time * 3.0).sin() * 0.2),
        ),
        E::Scan => (chosen, 0.0),
        E::Windmill => (rainbow(angle + time * 0.4), 1.0),
        E::Waterfall => (rainbow((y * 5.0).floor() / 5.0 - time * 0.4), 1.0),
        E::Blossom => (
            rainbow(radius * 2.0 - time * 0.3),
            pulse(radius * 3.0 - time * 0.6),
        ),
        E::Storm => (chosen, 0.0),
        E::Off => (chosen, 0.0),
    };
    color.gamma_multiply(strength.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reactive_modes_do_not_emit_without_a_key_event() {
        for effect in [E::Flash, E::Ripples, E::Fade, E::Reactive] {
            for time in [0.0, 0.7, 3.2, 10.0] {
                assert_eq!(sample(effect, 0.3, 0.5, time, 0.0, Color32::WHITE).r(), 0);
            }
        }
    }
    #[test]
    fn active_effects_have_spatial_and_temporal_signatures() {
        let effects = [
            E::Solid,
            E::Breathing,
            E::Rainbow,
            E::Flash,
            E::Rain,
            E::RainbowWheel,
            E::Ripples,
            E::Stars,
            E::Snake,
            E::Neon,
            E::Reactive,
            E::Sine,
            E::Waterfall,
            E::Blossom,
            E::Off,
        ];
        let mut signatures = std::collections::HashSet::new();
        for effect in effects {
            let mut signature = Vec::new();
            for t in [0.15, 0.65, 1.25] {
                for row in 0..4 {
                    for col in 0..14 {
                        signature.push(
                            sample(
                                effect,
                                col as f32 / 13.0,
                                row as f32 / 3.0,
                                t,
                                0.65,
                                Color32::from_rgb(0, 200, 255),
                            )
                            .to_array(),
                        );
                    }
                }
            }
            assert!(signatures.insert(signature), "duplicate effect: {effect:?}");
        }
    }
}
