use super::{ID, battery::Reading, descriptor, driver::lock_name, radio_codec as codec};
use crate::core::{
    Bus, Device, DeviceDescriptor, Discovery, DriverError, DriverInfo, Endpoint, Guard, HidChannel,
    Note,
};
use crate::profiles;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub(super) fn admits(endpoint: &Endpoint) -> bool {
    endpoint.vid == 0x3554
        && endpoint.pid == 0xfa09
        && endpoint.release == 0x2008
        && endpoint.usage_page == 0xff02
        && endpoint.usage == 2
        && endpoint.has_reports(20, 20)
        && endpoint.has_feature(0)
        && profiles::find(endpoint, ID).is_some()
}

struct Exchange {
    channel: Box<dyn HidChannel>,
    _lease: Box<dyn Guard>,
}

fn open(
    bus: &dyn Bus,
    endpoint: &Endpoint,
    validate_descriptor: bool,
) -> Result<Exchange, DriverError> {
    let lease = bus.lock(&lock_name(endpoint))?;
    let channel = bus.open_hid(endpoint)?;
    if validate_descriptor {
        let mut descriptor = [0; 128];
        let n = channel.report_descriptor(&mut descriptor)?;
        if descriptor.get(..n) != Some(codec::DESCRIPTOR) {
            return Err(DriverError::rejected("foreign radio report descriptor"));
        }
    }
    Ok(Exchange {
        channel,
        _lease: lease,
    })
}

fn read(channel: &dyn HidChannel) -> Result<Reading, DriverError> {
    let n = channel.write(&codec::request())?;
    if n != 0 && n != 20 {
        return Err(DriverError::protocol("short radio read request"));
    }
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut buffer = [0; 64];
    // Bounded receive only; never resend a request or create a background poller.
    for _ in 0..8 {
        if Instant::now() >= deadline {
            break;
        }
        let n = channel.read_timeout(&mut buffer, 350)?;
        if n == 0 {
            continue;
        }
        let packet = buffer
            .get(..n)
            .ok_or_else(|| DriverError::protocol("oversized radio reply"))?;
        if packet.first() == Some(&0x13) && packet.get(1) == Some(&0x4a) {
            return codec::reply(packet);
        }
    }
    Err(DriverError::Unavailable(
        "radio battery read timed out; wake the keyboard and search again".into(),
    ))
}

struct RadioDevice {
    descriptor: DeviceDescriptor,
    bus: Arc<dyn Bus>,
    endpoint: Endpoint,
}
impl Device for RadioDevice {
    fn battery(&mut self) -> Option<&mut dyn crate::core::Battery> {
        Some(self)
    }
    fn descriptor(&self) -> &DeviceDescriptor {
        &self.descriptor
    }
}

pub(super) fn discover(bus: &Arc<dyn Bus>, info: DriverInfo, found: &mut Discovery) {
    let endpoints: Vec<_> = bus.hid_endpoints().iter().filter(|e| admits(e)).collect();
    // Shared receiver IDs cannot distinguish multiple attached boards of this profile.
    if endpoints.len() != 1 {
        return;
    }
    let endpoint = endpoints[0];
    let exchange = match open(bus.as_ref(), endpoint, true) {
        Ok(exchange) => exchange,
        Err(error) => {
            found.notes.push(Note {
                subject: Some(endpoint.product.clone()),
                error,
            });
            return;
        }
    };
    let mut descriptor = descriptor(endpoint, info);
    match read(exchange.channel.as_ref()) {
        Ok(reading) => descriptor.details.extend(reading.details()),
        // Receiver identity is already validated. Telemetry failure must not remove its tile
        // or invent a battery percentage; the existing battery capability can recover later.
        Err(error) => found.notes.push(Note {
            subject: Some(endpoint.product.clone()),
            error,
        }),
    }
    found.devices.push(Box::new(RadioDevice {
        descriptor,
        bus: Arc::clone(bus),
        endpoint: endpoint.clone(),
    }));
}

impl crate::core::Battery for RadioDevice {
    fn refresh_battery(&mut self) -> Result<(), DriverError> {
        let exchange = open(self.bus.as_ref(), &self.endpoint, false)?;
        let reading = read(exchange.channel.as_ref())?;
        self.descriptor.details.retain(|d| {
            !matches!(
                d,
                crate::core::Detail::Battery(_) | crate::core::Detail::Power(_)
            )
        });
        self.descriptor.details.extend(reading.details());
        Ok(())
    }
}
