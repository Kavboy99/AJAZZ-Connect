//! Passive USB/HID diagnostics: what is connected, which family it looks like, and a JSON report
//! a contributor can share. Contains no serial numbers, device paths or key presses.

mod probe;
mod report;

pub use probe::probe;
pub use report::{Interface, Report};
