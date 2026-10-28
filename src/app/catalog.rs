//! Offline model groups. Display metadata never changes hardware admission.
use crate::core::{DeviceDescriptor, DriverInfo};
use crate::profiles::PreviewProfile;

/// Driver-supplied controls are shared by offline and unsupported-connection previews.
pub fn lighting_preview(driver: DriverInfo) -> Option<&'static crate::core::LightingPreview> {
    static PREVIEWS: std::sync::OnceLock<Vec<(&'static str, crate::core::LightingPreview)>> =
        std::sync::OnceLock::new();
    PREVIEWS
        .get_or_init(|| {
            crate::drivers::registry()
                .iter()
                .filter_map(|driver| {
                    driver
                        .lighting_preview()
                        .map(|preview| (driver.info().id, preview))
                })
                .collect()
        })
        .iter()
        .find(|(id, _)| *id == driver.id)
        .map(|(_, preview)| preview)
}

pub struct CatalogDevice {
    identity: PreviewProfile,
    variants: Vec<DeviceDescriptor>,
}

impl CatalogDevice {
    pub fn name(&self) -> &str {
        &self.identity.model
    }
    pub fn identity(&self) -> &PreviewProfile {
        &self.identity
    }
    pub fn variants(&self) -> &[DeviceDescriptor] {
        &self.variants
    }
    pub fn matches(&self, descriptor: &DeviceDescriptor) -> bool {
        self.variants.iter().any(|variant| {
            variant.vendor == descriptor.vendor
                && variant.kind == descriptor.kind
                && variant.driver.id == descriptor.driver.id
                && (self.name() == descriptor.name || variant.name == descriptor.name)
        })
    }
}

pub fn devices(drivers: &[DriverInfo]) -> Vec<CatalogDevice> {
    let mut devices: Vec<CatalogDevice> = Vec::new();
    for profile in crate::profiles::devices() {
        let Some(driver) = drivers
            .iter()
            .find(|d| Some(d.id) == profile.driver.as_deref())
        else {
            continue;
        };
        let name = profile.catalog_model.as_ref().unwrap_or(&profile.model);
        let descriptor = DeviceDescriptor {
            name: profile.model.clone(),
            vendor: Some(profile.vendor.clone()),
            kind: profile.kind,
            visual: profile.visual.clone(),
            driver: *driver,
            details: Vec::new(),
        };
        let group = devices.iter_mut().find(|group| {
            group.name() == *name
                && group.variants.first().is_some_and(|variant| {
                    variant.vendor == descriptor.vendor && variant.kind == descriptor.kind
                })
        });
        if let Some(group) = group {
            if !group.variants.iter().any(|v| v.driver.id == driver.id) {
                group.variants.push(descriptor);
            }
        } else {
            devices.push(CatalogDevice {
                identity: PreviewProfile {
                    model: name.clone(),
                    vendor: profile.vendor.clone(),
                    kind: profile.kind,
                    visual: profile.visual.clone(),
                },
                variants: vec![descriptor],
            });
        }
    }
    devices.extend(
        crate::profiles::previews()
            .iter()
            .cloned()
            .map(|identity| CatalogDevice {
                identity,
                variants: Vec::new(),
            }),
    );
    devices
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn placeholders_have_no_driver_or_live_device_match() {
        let catalog = devices(&[]);
        assert_eq!(catalog.len(), crate::profiles::previews().len());
        for group in catalog {
            assert!(group.variants().is_empty());
            assert!(group.identity().visual.is_some());
            for profile in crate::profiles::devices() {
                assert_ne!(group.name(), profile.model);
            }
        }
    }
    #[test]
    fn variants_are_grouped_without_losing_drivers() {
        let drivers: Vec<_> = crate::drivers::registry()
            .iter()
            .map(|d| d.info())
            .collect();
        let catalog = devices(&drivers);
        let count = crate::profiles::devices()
            .iter()
            .filter(|p| drivers.iter().any(|d| Some(d.id) == p.driver.as_deref()))
            // USB and radio profiles of one model share a driver variant.
            .map(|p| {
                (
                    p.catalog_model.as_ref().unwrap_or(&p.model),
                    &p.vendor,
                    format!("{:?}", p.kind),
                    &p.driver,
                )
            })
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        assert_eq!(
            catalog.iter().map(|d| d.variants().len()).sum::<usize>(),
            count
        );
        for group in &catalog {
            for variant in group.variants() {
                assert!(group.matches(variant));
                assert_eq!(
                    group
                        .variants()
                        .iter()
                        .filter(|v| v.driver.id == variant.driver.id)
                        .count(),
                    1
                );
            }
        }
        assert!(devices(&[]).iter().all(|group| group.variants().is_empty()));
    }

    #[test]
    fn explicit_model_names_share_one_card_and_unrelated_names_do_not_match() {
        let drivers: Vec<_> = crate::drivers::registry()
            .iter()
            .map(|d| d.info())
            .collect();
        let catalog = devices(&drivers);
        for profile in crate::profiles::devices()
            .iter()
            .filter(|p| drivers.iter().any(|d| Some(d.id) == p.driver.as_deref()))
        {
            let name = profile.catalog_model.as_ref().unwrap_or(&profile.model);
            assert_eq!(
                catalog
                    .iter()
                    .filter(|g| g.name() == name
                        && g.variants()
                            .first()
                            .is_some_and(|v| v.vendor.as_deref() == Some(profile.vendor.as_str())
                                && v.kind == profile.kind))
                    .count(),
                1
            );
        }
        for group in &catalog {
            for variant in group.variants() {
                let mut unrelated = variant.clone();
                unrelated.name = "Unrelated device".into();
                assert!(!group.matches(&unrelated));
            }
        }
    }
}
