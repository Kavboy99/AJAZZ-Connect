//! One folder per protocol. `registry()` is the only place where drivers are listed; adding a
//! driver needs no change anywhere else (`docs/ADDING-A-DRIVER.md`).

#[cfg(feature = "driver-by-tech")]
pub mod by_tech;
#[cfg(test)]
mod fake_bus;
#[cfg(feature = "driver-hid-lamparray")]
pub mod hid_lamparray;
#[cfg(feature = "driver-openrgb")]
pub mod openrgb;
#[cfg(feature = "driver-sonix-aa55")]
pub mod sonix_aa55;
#[cfg(feature = "driver-sonix-feature")]
pub mod sonix_feature;

use crate::core::Driver;

pub fn registry() -> Vec<Box<dyn Driver>> {
    vec![
        #[cfg(feature = "driver-sonix-aa55")]
        Box::new(sonix_aa55::Aa55Driver),
        #[cfg(feature = "driver-sonix-feature")]
        Box::new(sonix_feature::FeatureDriver),
        #[cfg(feature = "driver-hid-lamparray")]
        Box::new(hid_lamparray::LampArrayDriver),
        #[cfg(feature = "driver-openrgb")]
        Box::new(openrgb::OpenRgbDriver::new()),
        #[cfg(feature = "driver-by-tech")]
        Box::new(by_tech::ByTechDriver),
    ]
}
