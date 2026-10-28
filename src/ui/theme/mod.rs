//! The design system: tokens, type ramp, fonts and their application to egui.

pub mod fonts;
pub mod mode;
pub mod style;
pub mod text;
pub mod tokens;

pub use text::{Role, label, rich};
pub use tokens::{breakpoint, color, motion as motion_tokens, radius, size, space};
