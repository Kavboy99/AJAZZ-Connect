//! One search for devices on a fresh enumeration. Read-only by the driver contract. An automatic
//! search first compares the passive enumeration with the previous one and stops there when no
//! interface a driver cares about came or went, so plugging in an unrelated device sends nothing
//! to anything and keeps every device as it is.

use super::events::{Discovered, DriverNote};
use crate::core::{Bus, Detail, DeviceDescriptor, DeviceKind, Endpoint, Maturity};
use crate::drivers;
use crate::profiles;
use crate::transport::SystemBus;
use std::sync::Arc;

/// Every endpoint some driver admits, as `driver id / port fingerprint`, sorted. Passive.
fn admitted(endpoints: &[Endpoint]) -> Vec<String> {
    let mut keys: Vec<String> = drivers::registry()
        .iter()
        .flat_map(|driver| {
            let id = driver.info().id;
            endpoints
                .iter()
                .filter(|e| driver.admits(e))
                .map(move |e| format!("{id}/{}", e.fingerprint()))
        })
        .collect();
    keys.sort();
    keys
}

/// A device of a driver the user has not enabled: named from its profile, nothing is sent to it.
fn dormant(endpoint: &Endpoint, driver: crate::core::DriverInfo) -> DeviceDescriptor {
    let profile = profiles::describe(endpoint);
    DeviceDescriptor {
        kind: profile.map_or(DeviceKind::Other, |p| p.kind),
        vendor: profile.map(|p| p.vendor.clone()),
        name: profile.map_or_else(|| endpoint.product.trim().to_owned(), |p| p.model.clone()),
        driver,
        details: vec![Detail::Usb {
            vid: endpoint.vid,
            pid: endpoint.pid,
        }],
        visual: profile.and_then(|p| p.visual.clone()),
    }
}

/// `previous` is the admitted set of the last search: `None` forces a full search.
pub fn search(experimental: bool, previous: Option<&[String]>) -> Option<Discovered> {
    let system = SystemBus::new();
    let admitted = admitted(system.hid_endpoints());
    if previous == Some(admitted.as_slice()) {
        return None;
    }
    let mut notes: Vec<DriverNote> = system
        .hid_error()
        .map(|error| DriverNote {
            driver: None,
            subject: None,
            error: error.clone(),
        })
        .into_iter()
        .collect();
    let dormant_devices: Vec<DeviceDescriptor> = drivers::registry()
        .iter()
        .filter(|d| d.info().maturity == Maturity::Experimental && !experimental)
        .flat_map(|d| {
            let info = d.info();
            system
                .hid_endpoints()
                .iter()
                .filter(|e| d.admits(e))
                .map(move |e| dormant(e, info))
                .collect::<Vec<_>>()
        })
        .collect();
    let bus: Arc<dyn Bus> = Arc::new(system);
    let mut devices = Vec::new();
    for driver in drivers::registry() {
        let info = driver.info();
        if info.maturity == Maturity::Experimental && !experimental {
            continue;
        }
        let found = driver.discover(&bus);
        devices.extend(found.devices);
        notes.extend(found.notes.into_iter().map(|n| DriverNote {
            driver: Some(info),
            subject: n.subject,
            error: n.error,
        }));
    }
    Some(Discovered {
        devices,
        dormant: dormant_devices,
        notes,
        admitted,
    })
}
