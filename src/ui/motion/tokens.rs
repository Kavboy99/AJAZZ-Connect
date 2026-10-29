//! Spring tokens (`docs/DESIGN.md`, section 9.2). New motion values are added here only.
//! Overshoot above 3% and damping below 0.7 are not allowed.

use super::SpringSpec;

/// Hover, press, focus, row highlight, color change. 95% in ≈ 0.14 s.
pub const FAST: SpringSpec = SpringSpec {
    period: 0.18,
    damping: 1.0,
};
/// Switches, selection, expanding a group, popups. 95% in ≈ 0.23 s.
pub const STANDARD: SpringSpec = SpringSpec {
    period: 0.30,
    damping: 1.0,
};
/// Page and window transitions. 95% in ≈ 0.30 s.
pub const PAGE: SpringSpec = SpringSpec {
    period: 0.40,
    damping: 0.95,
};
/// Device editors: camera and panel travel settle without overshoot.
pub const WORKSPACE: SpringSpec = SpringSpec {
    period: 0.40,
    damping: 1.0,
};
/// Shared device model during opening: a restrained overshoot (about 0.6%).
pub const DEVICE_REVEAL: SpringSpec = SpringSpec {
    period: 0.52,
    damping: 0.85,
};
/// Sliders and markers after a gesture, ≈ 1.5% overshoot.
pub const RELEASE: SpringSpec = SpringSpec {
    period: 0.32,
    damping: 0.80,
};
/// A device model turning toward the pointer: light inertia, ≈ 0.2% overshoot.
pub const TILT: SpringSpec = SpringSpec {
    period: 0.5,
    damping: 0.9,
};
/// With Windows animations off: a short (≈ 100 ms) fade and color change only.
pub const REDUCED: SpringSpec = SpringSpec {
    period: 0.12,
    damping: 1.0,
};

/// Kinetic scrolling: velocity decays as `v₀·e^(−t/τ)`.
pub const SCROLL_DECAY: f32 = 0.3;
/// Kinetic scrolling stops below this speed, px/s.
pub const SCROLL_STOP: f32 = 20.0;
