//! Borrowed display data shared by live passports and driver-free catalog placeholders.
use crate::app::catalog::CatalogDevice;
use crate::core::{Detail, DeviceDescriptor, DeviceKind, DriverInfo};

pub(super) struct DeviceView<'a> {
    pub name: &'a str,
    pub vendor: Option<&'a str>,
    pub kind: DeviceKind,
    pub visual: Option<&'a str>,
    pub driver: Option<DriverInfo>,
    pub details: &'a [Detail],
}

impl<'a> From<&'a DeviceDescriptor> for DeviceView<'a> {
    fn from(d: &'a DeviceDescriptor) -> Self {
        Self {
            name: &d.name,
            vendor: d.vendor.as_deref(),
            kind: d.kind,
            visual: d.visual.as_deref(),
            driver: Some(d.driver),
            details: &d.details,
        }
    }
}

impl<'a> DeviceView<'a> {
    pub fn catalog(group: &'a CatalogDevice, selected: usize) -> Self {
        if let Some(d) = group.variants().get(selected) {
            return d.into();
        }
        let identity = group.identity();
        Self {
            name: &identity.model,
            vendor: Some(&identity.vendor),
            kind: identity.kind,
            visual: identity.visual.as_deref(),
            driver: None,
            details: &[],
        }
    }
}
