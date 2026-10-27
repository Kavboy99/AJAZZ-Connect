pub mod aa55;
pub mod families;
pub mod firmware_assistant;
#[cfg(windows)]
mod hardware;
pub mod lamparray;
pub mod lighting;
pub mod openrgb;
pub mod protocol;
pub mod sonix_feature;
pub mod storage;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
