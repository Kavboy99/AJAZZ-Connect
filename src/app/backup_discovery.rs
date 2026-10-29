//! Read-only backup discovery, including passive placeholders for unknown keyboard interfaces.
use super::events::{Discovered, DriverNote};
use crate::{
    core::{Bus, DeviceDescriptor, DeviceKind, DriverInfo, Maturity},
    drivers,
    transport::SystemBus,
};
use std::{collections::BTreeSet, sync::Arc};

pub(super) fn search() -> Discovered {
    let system = SystemBus::new();
    let mut notes = Vec::new();
    if let Some(error) = system.hid_error() {
        notes.push(DriverNote {
            driver: None,
            subject: None,
            error: error.clone(),
        });
    }
    search_bus(Arc::new(system), notes)
}

fn search_bus(bus: Arc<dyn Bus>, mut notes: Vec<DriverNote>) -> Discovered {
    let registry: Vec<_> = drivers::registry()
        .into_iter()
        .filter(|d| d.supports_configuration_backup())
        .collect();
    let mut known_containers = BTreeSet::new();
    for endpoint in bus
        .hid_endpoints()
        .iter()
        .filter(|e| registry.iter().any(|d| d.admits(e)))
    {
        if let Ok(channel) = bus.open_hid(endpoint)
            && let Ok(id) = channel.container_id()
        {
            known_containers.insert(id);
        }
    }
    let mut devices = Vec::new();
    for driver in &registry {
        let found = driver.discover(&bus);
        devices.extend(found.devices);
        notes.extend(found.notes.into_iter().map(|n| DriverNote {
            driver: Some(driver.info()),
            subject: n.subject,
            error: n.error,
        }));
    }
    let mut dormant = Vec::new();
    let mut seen = known_containers;
    for endpoint in bus
        .hid_endpoints()
        .iter()
        .filter(|e| e.usage_page == 1 && e.usage == 6)
    {
        let container = bus
            .open_hid(endpoint)
            .ok()
            .and_then(|c| c.container_id().ok());
        if container.is_some_and(|id| !seen.insert(id)) {
            continue;
        }
        dormant.push(DeviceDescriptor {
            name: endpoint.product.clone(),
            vendor: None,
            kind: DeviceKind::Keyboard,
            visual: None,
            driver: DriverInfo {
                id: "unknown_backup",
                name: "HID",
                maturity: Maturity::Experimental,
                requirement: None,
            },
            details: vec![],
        });
    }
    Discovered {
        devices,
        dormant,
        notes,
        admitted: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{core::Endpoint, drivers::fake_bus::FakeBus};
    #[test]
    fn unknown_keyboard_is_visible_without_any_protocol_request() {
        let endpoint = Endpoint::hid(0xabcd, 0x1234, 1, 6).with_product("Unknown keyboard");
        let bus = FakeBus::new(
            vec![endpoint.clone(), endpoint],
            Box::new(|_| panic!("unknown devices must receive no packets")),
        );
        let result = search_bus(bus.clone(), vec![]);
        assert!(result.devices.is_empty());
        assert_eq!(result.dormant.len(), 1);
        assert_eq!(result.dormant[0].name, "Unknown keyboard");
        assert!(bus.log().is_empty());
    }
}
