use super::codec::{self, ATTRIBUTES_BYTES, LAMP_ATTRIBUTES_BYTES};
use super::descriptor::{ReportMap, parse_descriptor};
use super::{ID, admits};
use crate::app::identity;
use crate::core::{
    Bus, Detail, Device, DeviceDescriptor, Discovery, Driver, DriverError, DriverInfo, Endpoint,
    HidChannel, Identification, Lighting, LightingTraits, Maturity, Note, Outcome, Rgb, Rollback,
    Snapshot,
};
use std::sync::Arc;
use std::time::Duration;

const INFO: DriverInfo = DriverInfo {
    id: ID,
    name: "HID LampArray",
    maturity: Maturity::Stable,
    requirement: None,
};

const DESCRIPTOR_LIMIT: usize = 16384;

pub struct LampArrayDriver;

impl Driver for LampArrayDriver {
    fn info(&self) -> DriverInfo {
        INFO
    }

    fn admits(&self, endpoint: &Endpoint) -> bool {
        admits(endpoint)
    }

    fn discover(&self, bus: &Arc<dyn Bus>) -> Discovery {
        let mut found = Discovery::default();
        for endpoint in bus.hid_endpoints().iter().filter(|e| admits(e)) {
            match LampArrayDevice::confirm(Arc::clone(bus), endpoint.clone()) {
                Ok(device) => found.devices.push(Box::new(device)),
                Err(error) => found.notes.push(Note {
                    subject: Some(endpoint.product.clone()),
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

fn feature(channel: &dyn HidChannel, id: u8, size: usize) -> Result<Vec<u8>, DriverError> {
    let mut p = vec![0; size + 1];
    p[0] = id;
    let n = channel.get_feature(&mut p)?;
    if n != p.len() || p[0] != id {
        return Err(DriverError::protocol("truncated or foreign Feature report"));
    }
    p.remove(0);
    Ok(p)
}

pub(super) struct LampArrayDevice {
    bus: Arc<dyn Bus>,
    endpoint: Endpoint,
    descriptor: DeviceDescriptor,
    map: ReportMap,
    lamps: u16,
    interval_us: u32,
    lock: String,
    /// Autonomous mode is off: the device shows what the application sent.
    controlled: bool,
}

impl LampArrayDevice {
    /// Reads the descriptor once and the attributes report. Sends nothing that changes lighting.
    fn confirm(bus: Arc<dyn Bus>, endpoint: Endpoint) -> Result<Self, DriverError> {
        let channel = bus.open_hid(&endpoint)?;
        let mut buffer = vec![0; DESCRIPTOR_LIMIT];
        let n = channel.report_descriptor(&mut buffer)?;
        if n >= buffer.len() {
            return Err(DriverError::protocol("HID descriptor truncated"));
        }
        let map = parse_descriptor(&buffer[..n])?;
        let attributes = codec::parse_attributes(&feature(
            channel.as_ref(),
            map.attributes,
            ATTRIBUTES_BYTES,
        )?)?;
        drop(channel);
        let fingerprint = endpoint.fingerprint();
        let name = endpoint.product.trim();
        Ok(Self {
            descriptor: DeviceDescriptor {
                kind: attributes.kind,
                vendor: None,
                name: if name.is_empty() { "LampArray" } else { name }.to_owned(),
                driver: INFO,
                details: vec![
                    Detail::Usb {
                        vid: endpoint.vid,
                        pid: endpoint.pid,
                    },
                    Detail::Leds(attributes.lamps as usize),
                ],
                visual: None,
            },
            lock: format!("Local\\{}-{}", identity::LOCK_PREFIX, &fingerprint[..16]),
            bus,
            endpoint,
            map,
            lamps: attributes.lamps,
            interval_us: attributes.interval_us,
            controlled: false,
        })
    }

    /// Querying lamp attributes is a standard read; it changes neither lighting nor memory.
    fn programmable(
        &self,
        channel: &dyn HidChannel,
        rgb: [u8; 3],
    ) -> Result<Vec<(u16, [u8; 4])>, DriverError> {
        let mut lamps = Vec::new();
        for id in 0..self.lamps {
            let mut request = vec![self.map.request];
            request.extend_from_slice(&id.to_le_bytes());
            channel.send_feature(&request)?;
            let reply = feature(channel, self.map.response, LAMP_ATTRIBUTES_BYTES)?;
            if let Some(levels) = codec::parse_lamp(&reply, id)? {
                lamps.push((id, codec::lamp_color(rgb, levels)));
            }
        }
        if lamps.is_empty() {
            return Err(DriverError::rejected("no programmable lamps"));
        }
        Ok(lamps)
    }

    fn send(&self, channel: &dyn HidChannel, lamps: &[(u16, [u8; 4])]) -> Result<(), DriverError> {
        let chunks = lamps.len().div_ceil(self.map.capacity);
        for (i, group) in lamps.chunks(self.map.capacity).enumerate() {
            channel.send_feature(&codec::multi_packet(&self.map, group, i + 1 == chunks)?)?;
            std::thread::sleep(Duration::from_micros(self.interval_us.max(1000) as u64));
        }
        Ok(())
    }

    fn release(&mut self, channel: &dyn HidChannel) -> Result<(), DriverError> {
        channel.send_feature(&codec::control(&self.map, true))?;
        self.controlled = false;
        Ok(())
    }
}

impl Device for LampArrayDevice {
    fn descriptor(&self) -> &DeviceDescriptor {
        &self.descriptor
    }

    fn lighting(&mut self) -> Option<&mut dyn Lighting> {
        Some(self)
    }
}

impl Lighting for LampArrayDevice {
    fn traits(&self) -> LightingTraits {
        LightingTraits {
            writable: true,
            persistent: false,
            rollback: Rollback::Session,
        }
    }

    fn apply_solid(&mut self, color: Rgb) -> Result<Outcome, DriverError> {
        let _lease = self.bus.lock(&self.lock)?;
        let channel = self.bus.open_hid(&self.endpoint)?;
        let lamps = self.programmable(channel.as_ref(), color.to_array())?;
        channel.send_feature(&codec::control(&self.map, false))?;
        self.controlled = true;
        if let Err(error) = self.send(channel.as_ref(), &lamps) {
            let _ = self.release(channel.as_ref());
            return Err(error);
        }
        Ok(Outcome::Sent { lamps: lamps.len() })
    }

    fn restore(&mut self) -> Result<Outcome, DriverError> {
        if self.controlled {
            let channel = self.bus.open_hid(&self.endpoint)?;
            self.release(channel.as_ref())?;
        }
        Ok(Outcome::Released)
    }

    fn restore_snapshot(&mut self, _: &Snapshot) -> Result<Outcome, DriverError> {
        Err(DriverError::NoRollback)
    }
}

/// Closing the application hands the lighting back to the device.
impl Drop for LampArrayDevice {
    fn drop(&mut self) {
        if self.controlled {
            let _ = self.restore();
        }
    }
}
