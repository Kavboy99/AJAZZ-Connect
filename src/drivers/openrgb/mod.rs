//! Client of the published OpenRGB SDK (protocol levels 1..3) on the loopback interface only.
//! Independent implementation; no SaveMode, firmware commands, scanning or server launch.

mod backup;
mod client;
mod driver;
#[cfg(test)]
mod tests;
pub mod wire;

pub use driver::OpenRgbDriver;

pub const ID: &str = "openrgb";
