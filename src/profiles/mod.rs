//! Device data, not code. Built into the executable at compile time and parsed once, on first
//! use. A profile is a hint for choosing a driver and naming a device, never a permission to write:
//! admission is still checked by the driver.

mod rule;

use crate::core::{DeviceKind, Endpoint};
use serde::Deserialize;
use std::sync::OnceLock;

pub use rule::Rule;

const SOURCES: &[(&str, &str)] = &[
    ("sonix.toml", include_str!("data/sonix.toml")),
    ("by_tech.toml", include_str!("data/by_tech.toml")),
    ("families.toml", include_str!("data/families.toml")),
];

/// How far support for a device has come (`docs/DEVICES.md`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Support {
    Identified,
    Read,
    WrittenByOwner,
    Confirmed,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceProfile {
    pub vendor: String,
    pub model: String,
    pub kind: DeviceKind,
    pub driver: Option<String>,
    pub support: Support,
    vid: u16,
    #[serde(default)]
    pids: Vec<u16>,
    product_contains: Option<String>,
    #[serde(default)]
    product_names: Vec<String>,
    /// A 3D model built into the interface (`ui/viewer/catalog.rs`).
    pub visual: Option<String>,
}

impl DeviceProfile {
    pub fn matches(&self, endpoint: &Endpoint) -> bool {
        let product = endpoint.product.trim();
        endpoint.vid == self.vid
            && (self.pids.is_empty() || self.pids.contains(&endpoint.pid))
            && self
                .product_contains
                .as_deref()
                .is_none_or(|part| product.to_ascii_uppercase().contains(part))
            && (self.product_names.is_empty() || self.product_names.iter().any(|n| n == product))
    }
}

/// Text in every interface language. English goes to diagnostic reports.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Localized {
    pub en: String,
    pub ru: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Family {
    pub name: String,
    pub hint: Localized,
    #[serde(flatten)]
    rule: Rule,
}

impl Family {
    pub fn matches(&self, endpoint: &Endpoint) -> bool {
        self.rule.matches(endpoint)
    }
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(default)]
    device: Vec<DeviceProfile>,
    #[serde(default)]
    family: Vec<Family>,
}

#[derive(Default)]
struct Profiles {
    devices: Vec<DeviceProfile>,
    families: Vec<Family>,
}

fn parse(text: &str) -> Result<File, toml::de::Error> {
    toml::from_str(text)
}

fn profiles() -> &'static Profiles {
    static PROFILES: OnceLock<Profiles> = OnceLock::new();
    PROFILES.get_or_init(|| {
        let mut all = Profiles::default();
        // Built-in data is checked by `every_built_in_file_parses`; a broken file adds nothing.
        for file in SOURCES.iter().filter_map(|(_, text)| parse(text).ok()) {
            all.devices.extend(file.device);
            all.families.extend(file.family);
        }
        all
    })
}

pub fn devices() -> &'static [DeviceProfile] {
    &profiles().devices
}

/// Families in priority order: the first match explains an interface.
pub fn families() -> &'static [Family] {
    &profiles().families
}

/// The profile that names `endpoint` for `driver`, if any.
pub fn find(endpoint: &Endpoint, driver: &str) -> Option<&'static DeviceProfile> {
    devices()
        .iter()
        .find(|p| p.driver.as_deref() == Some(driver) && p.matches(endpoint))
}

/// The first profile that names `endpoint`, with or without a driver.
pub fn describe(endpoint: &Endpoint) -> Option<&'static DeviceProfile> {
    devices().iter().find(|p| p.matches(endpoint))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_built_in_file_parses() {
        for (name, text) in SOURCES {
            if let Err(error) = parse(text) {
                panic!("{name}: {error}");
            }
        }
        assert!(devices().len() >= 4);
        assert!(families().len() >= 12);
    }

    #[test]
    fn product_filter_is_case_insensitive_and_exact_names_are_exact() {
        let aa55 = Endpoint::hid(0x0c45, 0x80a0, 0xff68, 0x61).with_product("ajazz ak820 max");
        assert_eq!(
            find(&aa55, "sonix_aa55").map(|p| p.vendor.as_str()),
            Some("AJAZZ")
        );
        let feature = Endpoint::hid(0x0c45, 0x8009, 0xff13, 1).with_product(" AK820 MAX ");
        assert!(find(&feature, "sonix_feature").is_some());
        let other = Endpoint::hid(0x0c45, 0x8009, 0xff13, 1).with_product("AK820 MAX Pro");
        assert!(find(&other, "sonix_feature").is_none());
    }
}
