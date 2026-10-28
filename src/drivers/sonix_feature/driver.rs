use super::codec::{self, FINISH, REPORT_BYTES, START};
use super::{ID, VENDOR_LOCK, admits, descriptor};
use crate::core::{
    Bus, Detail, Device, DeviceDescriptor, Discovery, Driver, DriverError, DriverInfo, Endpoint,
    HidChannel, Identification, Lighting, LightingTraits, Maturity, Note, Outcome, Rgb, Rollback,
    Snapshot,
};
use std::sync::Arc;
use std::time::Duration;

const INFO: DriverInfo = DriverInfo {
    id: ID,
    name: "SONiX Feature",
    maturity: Maturity::Experimental,
    requirement: None,
};

/// The firmware needs a pause between Feature reports.
const PACE: Duration = Duration::from_millis(35);

pub struct FeatureDriver;

impl Driver for FeatureDriver {
    fn info(&self) -> DriverInfo {
        INFO
    }

    fn admits(&self, endpoint: &Endpoint) -> bool {
        admits(endpoint)
    }

    fn discover(&self, bus: &Arc<dyn Bus>) -> Discovery {
        let mut found = Discovery::default();
        let candidates: Vec<Endpoint> = bus
            .hid_endpoints()
            .iter()
            .filter(|e| admits(e))
            .cloned()
            .collect();
        if candidates.is_empty() {
            return found;
        }
        let _lease = match bus.lock(VENDOR_LOCK) {
            Ok(lease) => lease,
            Err(error) => {
                found.notes.push(Note {
                    subject: None,
                    error,
                });
                return found;
            }
        };
        for endpoint in candidates {
            let subject = endpoint.product.clone();
            match FeatureDevice::confirm(Arc::clone(bus), endpoint) {
                Ok(device) => found.devices.push(Box::new(device)),
                Err(error) => found.notes.push(Note {
                    subject: Some(subject),
                    error,
                }),
            }
        }
        found
    }

    fn identify(&self, _: &dyn Bus, _: &Endpoint) -> Option<Result<Identification, DriverError>> {
        None
    }
}

fn send(
    channel: &dyn HidChannel,
    p: &[u8; REPORT_BYTES],
    handshake: bool,
) -> Result<(), DriverError> {
    std::thread::sleep(PACE);
    channel.send_feature(p)?;
    if handshake {
        std::thread::sleep(PACE);
        let mut response = [0; REPORT_BYTES];
        if channel.get_feature(&mut response)? != REPORT_BYTES {
            return Err(DriverError::protocol("short Feature report"));
        }
        codec::ack(&response, p[2])?;
    }
    Ok(())
}

/// The large data pipe of the same family must sit in the same physical USB device.
fn has_data_partner(bus: &dyn Bus, endpoint: &Endpoint, container: u128) -> bool {
    bus.hid_endpoints()
        .iter()
        .filter(|d| {
            d.vid == endpoint.vid
                && d.pid == endpoint.pid
                && d.usage_page == 0xff68
                && d.usage == 0x61
                && d.reports.is_some_and(|r| r.output == 4097)
        })
        .any(|d| {
            bus.open_hid(d)
                .and_then(|channel| channel.container_id())
                .is_ok_and(|id| id == container)
        })
}

pub(super) struct FeatureDevice {
    bus: Arc<dyn Bus>,
    endpoint: Endpoint,
    descriptor: DeviceDescriptor,
}

impl FeatureDevice {
    /// Checks the USB container, then a START/FINISH pair that changes nothing.
    fn confirm(bus: Arc<dyn Bus>, endpoint: Endpoint) -> Result<Self, DriverError> {
        let channel = bus.open_hid(&endpoint)?;
        let container = channel.container_id()?;
        if !has_data_partner(bus.as_ref(), &endpoint, container) {
            return Err(DriverError::rejected(
                "no FF68/4096 pipe in the same USB container",
            ));
        }
        send(channel.as_ref(), &codec::command(START, 0)?, true)?;
        send(channel.as_ref(), &codec::command(FINISH, 0)?, false)?;
        drop(channel);
        let mut descriptor = descriptor(&endpoint, INFO);
        descriptor.details.push(Detail::Release(endpoint.release));
        Ok(Self {
            bus,
            endpoint,
            descriptor,
        })
    }
}

impl Device for FeatureDevice {
    fn descriptor(&self) -> &DeviceDescriptor {
        &self.descriptor
    }

    fn lighting(&mut self) -> Option<&mut dyn Lighting> {
        Some(self)
    }
}

impl Lighting for FeatureDevice {
    fn traits(&self) -> LightingTraits {
        LightingTraits {
            writable: true,
            persistent: true,
            rollback: Rollback::Unavailable,
        }
    }

    fn apply_solid(&mut self, color: Rgb) -> Result<Outcome, DriverError> {
        let _lease = self.bus.lock(VENDOR_LOCK)?;
        let channel = self.bus.open_hid(&self.endpoint)?;
        codec::apply_transaction(
            |p, handshake| send(channel.as_ref(), p, handshake),
            color.to_array(),
        )?;
        Ok(Outcome::Acknowledged)
    }

    /// The protocol has no confirmed command to read the original effect back.
    fn restore(&mut self) -> Result<Outcome, DriverError> {
        Err(DriverError::NoRollback)
    }

    fn restore_snapshot(&mut self, _: &Snapshot) -> Result<Outcome, DriverError> {
        Err(DriverError::NoRollback)
    }
}
