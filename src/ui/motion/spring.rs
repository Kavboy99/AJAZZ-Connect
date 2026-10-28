//! Damped spring solved analytically, so any time step is stable and a stalled frame cannot blow
//! the motion up. State is position, velocity and target; changing the target keeps both.

use std::f32::consts::TAU;

/// Period `T` in seconds and damping ratio `ζ` (`docs/DESIGN.md`, section 9.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringSpec {
    pub period: f32,
    pub damping: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Spring {
    value: f32,
    velocity: f32,
    target: f32,
}

impl Spring {
    pub fn at(value: f32) -> Self {
        Self {
            value,
            velocity: 0.0,
            target: value,
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Jumps to the target and stops.
    pub fn settle(&mut self) {
        self.value = self.target;
        self.velocity = 0.0;
    }

    pub fn step(&mut self, spec: SpringSpec, dt: f32) {
        let omega = TAU / spec.period;
        let x0 = self.value - self.target;
        let v0 = self.velocity;
        let (x, v) = if spec.damping >= 1.0 {
            // Critically damped: (A + B t) e^(-ωt).
            let b = v0 + omega * x0;
            let decay = (-omega * dt).exp();
            ((x0 + b * dt) * decay, (v0 - omega * b * dt) * decay)
        } else {
            let a = spec.damping * omega;
            let wd = omega * (1.0 - spec.damping * spec.damping).sqrt();
            let decay = (-a * dt).exp();
            let (sin, cos) = (wd * dt).sin_cos();
            (
                decay * (x0 * cos + (v0 + a * x0) / wd * sin),
                decay * (v0 * cos - (a * v0 + omega * omega * x0) / wd * sin),
            )
        };
        self.value = self.target + x;
        self.velocity = v;
    }

    /// At rest when it is within `epsilon` of the target and would move less than `epsilon` in
    /// the next 60 Hz frame.
    pub fn at_rest(&self, epsilon: f32) -> bool {
        (self.value - self.target).abs() < epsilon && self.velocity.abs() / 60.0 < epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::super::tokens::{FAST, PAGE, RELEASE, STANDARD};
    use super::*;

    fn time_to(spec: SpringSpec, fraction: f32) -> f32 {
        let mut s = Spring::at(0.0);
        s.set_target(1.0);
        let dt = 1.0 / 240.0;
        let mut t = 0.0;
        while s.value() < fraction {
            s.step(spec, dt);
            t += dt;
        }
        t
    }

    #[test]
    fn tokens_reach_95_percent_as_documented() {
        assert!((time_to(FAST, 0.95) - 0.14).abs() < 0.02);
        assert!((time_to(STANDARD, 0.95) - 0.23).abs() < 0.02);
        assert!((time_to(PAGE, 0.95) - 0.30).abs() < 0.03);
        assert!(time_to(RELEASE, 0.95) < 0.35);
    }

    #[test]
    fn overshoot_stays_within_three_percent() {
        for spec in [
            FAST,
            STANDARD,
            PAGE,
            RELEASE,
            super::super::tokens::DEVICE_REVEAL,
        ] {
            let mut s = Spring::at(0.0);
            s.set_target(1.0);
            let mut peak: f32 = 0.0;
            for _ in 0..600 {
                s.step(spec, 1.0 / 120.0);
                peak = peak.max(s.value());
            }
            assert!(peak < 1.03, "{spec:?}: {peak}");
            assert!(s.at_rest(0.005));
        }
    }

    #[test]
    fn large_steps_are_stable_and_retargeting_keeps_velocity() {
        let mut s = Spring::at(0.0);
        s.set_target(100.0);
        s.step(STANDARD, 5.0);
        assert!((s.value() - 100.0).abs() < 1e-3);
        let mut moving = Spring::at(0.0);
        moving.set_target(1.0);
        moving.step(STANDARD, 0.05);
        let before = moving.velocity;
        moving.set_target(-1.0);
        assert_eq!(moving.velocity, before);
    }
}
