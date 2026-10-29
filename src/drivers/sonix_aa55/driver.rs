use super::backup::Backup;
mod configuration;
use super::codec::{self, GET_INFO, GET_LED_EFFECT, REPORT_BYTES, SET_LED_EFFECT};
use super::info::KeyboardInfo;
use super::{ID, VENDOR_LOCK, admits, descriptor};
use crate::core::{
    Bus, Detail, Device, DeviceDescriptor, Discovery, Driver, DriverError, DriverInfo, Endpoint,
    Guard, HidChannel, Identification, Lighting, LightingTraits, Maturity, Note, Outcome, Rgb,
    Rollback, Snapshot,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

const INFO: DriverInfo = DriverInfo {
    id: ID,
    name: "SONiX AA55",
    maturity: Maturity::Stable,
    requirement: None,
};

pub struct Aa55Driver;

impl Driver for Aa55Driver {
    fn supports_configuration_backup(&self) -> bool {
        true
    }
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
            match Aa55Device::confirm(Arc::clone(bus), endpoint) {
                Ok(device) => found.devices.push(Box::new(device)),
                Err(error) => found.notes.push(Note {
                    subject: Some(subject),
                    error,
                }),
            }
        }
        found
    }

    fn identify(
        &self,
        bus: &dyn Bus,
        endpoint: &Endpoint,
    ) -> Option<Result<Identification, DriverError>> {
        admits(endpoint).then(|| {
            let _lease = bus.lock(VENDOR_LOCK)?;
            let channel = bus.open_hid(endpoint)?;
            let raw = transaction(channel.as_ref(), GET_INFO, None)?;
            let info = KeyboardInfo::parse(&raw)?;
            Ok(Identification {
                driver: ID.into(),
                firmware_version: Some(info.version.clone()),
                reported_vid: Some(info.vid),
                reported_pid: Some(info.pid),
                details: serde_json::to_value(&info).unwrap_or_default(),
                payload_hex: raw.iter().map(|b| format!("{b:02x}")).collect(),
            })
        })
    }
}

/// Queues a native input read before the Output report, as the vendor WebHID transport does, then
/// waits for the echo. A write without an echo is reported and never repeated.
fn transaction(
    channel: &dyn HidChannel,
    command: u8,
    payload: Option<&[u8]>,
) -> Result<Vec<u8>, DriverError> {
    crate::diagnostics::logging::command(ID, command, payload.is_some());
    let mut quiet = false;
    for _ in 0..64 {
        let mut b = [0; REPORT_BYTES];
        if channel.read_timeout(&mut b, 0)? == 0 {
            quiet = true;
            break;
        }
    }
    if !quiet {
        return Err(DriverError::Busy);
    }
    let p = codec::packet(command, payload)?;
    let written = channel.write(&p)?;
    // The Windows native backend may return zero after a synchronous completion.
    if written != 0 && written != p.len() {
        return Err(DriverError::Io("incomplete output report".into()));
    }
    let size = codec::payload_size(command);
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        let mut b = [0; REPORT_BYTES];
        let n = channel.read_timeout(&mut b, 100)?;
        if let Ok(data) = codec::response(&b[..n], command, size) {
            return Ok(data);
        }
    }
    Err(DriverError::Unconfirmed(format!(
        "no reply to command {command:02X}"
    )))
}

/// One operation's hold on the device; the channel closes before the lock is released.
struct Exchange {
    channel: Box<dyn HidChannel>,
    _lease: Box<dyn Guard>,
}

pub(super) struct Aa55Device {
    bus: Arc<dyn Bus>,
    endpoint: Endpoint,
    descriptor: DeviceDescriptor,
    info: KeyboardInfo,
    identity: String,
    protocol_identity: String,
    original: Vec<u8>,
    backup: Option<Backup>,
}

impl Aa55Device {
    /// Confirms the family with GET_INFO and reads the current effect. Sends no write.
    fn confirm(bus: Arc<dyn Bus>, endpoint: Endpoint) -> Result<Self, DriverError> {
        let channel = bus.open_hid(&endpoint)?;
        let raw = transaction(channel.as_ref(), GET_INFO, None)?;
        let info = KeyboardInfo::parse(&raw)?;
        if info.vid != endpoint.vid || info.pid != endpoint.pid {
            return Err(DriverError::rejected("GET_INFO VID/PID differs from USB"));
        }
        let protocol_identity = codec::identity(&raw)?;
        let original = transaction(channel.as_ref(), GET_LED_EFFECT, None)?;
        codec::validate_lighting(&original)?;
        drop(channel);
        let mut descriptor = descriptor(&endpoint, INFO);
        descriptor
            .details
            .push(Detail::Firmware(info.version.clone()));
        descriptor.details.push(Detail::Battery(info.battery));
        Ok(Self {
            identity: endpoint.fingerprint(),
            bus,
            endpoint,
            descriptor,
            info,
            protocol_identity,
            original,
            backup: None,
        })
    }

    fn exchange(&self) -> Result<Exchange, DriverError> {
        let lease = self.bus.lock(VENDOR_LOCK)?;
        let channel = self.bus.open_hid(&self.endpoint)?;
        let exchange = Exchange {
            channel,
            _lease: lease,
        };
        let info = transaction(exchange.channel.as_ref(), GET_INFO, None)?;
        if codec::identity(&info)? != self.protocol_identity {
            return Err(DriverError::Changed);
        }
        Ok(exchange)
    }

    fn write_lighting(&self, channel: &dyn HidChannel, p: &[u8]) -> Result<Outcome, DriverError> {
        codec::validate_lighting(p)?;
        transaction(channel, SET_LED_EFFECT, Some(&codec::wire_form(p)))?;
        std::thread::sleep(Duration::from_millis(200));
        let observed = transaction(channel, GET_LED_EFFECT, None)?;
        codec::validate_lighting(&observed)?;
        if codec::same_state(&observed, p) {
            Ok(Outcome::Confirmed)
        } else if codec::palette_quirk(&self.info, p, &observed) {
            Ok(Outcome::ModeConfirmed)
        } else {
            Err(DriverError::ReadBackMismatch)
        }
    }

    fn restored(outcome: Outcome) -> Outcome {
        match outcome {
            Outcome::Confirmed => Outcome::Restored,
            other => other,
        }
    }
}

impl Device for Aa55Device {
    fn configuration_backup(&mut self) -> Option<&mut dyn crate::core::ConfigurationBackup> {
        Some(self)
    }
    fn descriptor(&self) -> &DeviceDescriptor {
        &self.descriptor
    }

    fn lighting(&mut self) -> Option<&mut dyn Lighting> {
        Some(self)
    }
}

impl Lighting for Aa55Device {
    fn traits(&self) -> LightingTraits {
        LightingTraits {
            writable: true,
            persistent: true,
            rollback: Rollback::SessionAndFile,
        }
    }

    fn apply_solid(&mut self, color: Rgb) -> Result<Outcome, DriverError> {
        let exchange = self.exchange()?;
        let channel = exchange.channel.as_ref();
        let current = transaction(channel, GET_LED_EFFECT, None)?;
        let p = codec::solid_payload(&current, color.to_array())?;
        if self.backup.is_none() {
            let backup = Backup::new(&self.identity, &self.protocol_identity, current.clone());
            self.bus
                .store_backup("aa55", &self.identity, &backup.to_json()?)?;
            self.original = current;
            self.backup = Some(backup);
        }
        self.write_lighting(channel, &p)
    }

    fn restore(&mut self) -> Result<Outcome, DriverError> {
        if self.backup.is_none() {
            return Err(DriverError::NoRollback);
        }
        let exchange = self.exchange()?;
        self.write_lighting(exchange.channel.as_ref(), &self.original)
            .map(Self::restored)
    }

    fn restore_snapshot(&mut self, snapshot: &Snapshot) -> Result<Outcome, DriverError> {
        crate::core::ConfigurationBackup::recover(self, snapshot)
    }
}
