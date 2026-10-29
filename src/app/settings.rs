use super::identity;
use crate::storage::atomic;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Interface theme. Dark is the primary one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub fn toggled(self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }
}

/// User preferences, stored as a small JSON file in the data directory.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Drivers implemented from descriptions and not verified on a device take part in search.
    pub experimental_drivers: bool,
    #[serde(skip)]
    pub ephemeral: bool,
    pub lighting: std::collections::BTreeMap<String, Vec<crate::core::LightingSettings>>,
    pub lighting_selected: std::collections::BTreeMap<String, u8>,
    pub theme: Theme,
    pub orange_keycaps: bool,
    pub developer_mode: bool,
    pub logging_enabled: bool,
    pub show_all_devices: bool,
    pub battery_mode: super::battery::BatteryMode,
}

fn path() -> Option<PathBuf> {
    identity::data_dir().map(|dir| dir.join("settings.json"))
}

impl Settings {
    /// Missing or unreadable settings fall back to the safe defaults.
    pub fn load() -> Self {
        path()
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        if self.ephemeral {
            return Ok(());
        }
        let path = path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "LOCALAPPDATA is not set")
        })?;
        atomic::write_json(&path, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn logging_is_opt_in_and_survives_settings_serialization() -> Result<(), serde_json::Error> {
        let old: Settings = serde_json::from_str(r#"{"developer_mode":true}"#)?;
        assert!(!old.logging_enabled);
        let configured = Settings {
            logging_enabled: true,
            ..old
        };
        let loaded: Settings = serde_json::from_slice(&serde_json::to_vec(&configured)?)?;
        assert!(loaded.logging_enabled);
        Ok(())
    }
}
