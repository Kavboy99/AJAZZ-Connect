//! The current theme as one number: 0 is dark, 1 is light. The application moves it with a spring
//! when the user switches themes, so every token flows to its new value together.

use std::sync::atomic::{AtomicU32, Ordering};

/// `f32` bits; starts at 0.0 (dark, the primary theme).
static BLEND: AtomicU32 = AtomicU32::new(0);

pub fn blend() -> f32 {
    f32::from_bits(BLEND.load(Ordering::Relaxed))
}

pub fn set_blend(t: f32) {
    BLEND.store(t.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
}

/// The theme the interface is closer to; egui's own widgets follow it.
pub fn is_dark() -> bool {
    blend() < 0.5
}
