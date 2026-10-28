use super::backup::Backup;
use super::codec::{self, Command, PASSWORDS, REPORT_BYTES, REPORT_ID};
use super::state::{ColorTable, Config};
use super::{ID, admits, descriptor};
use crate::app::identity;
use crate::core::{
    Bus, Device, DeviceDescriptor, Discovery, Driver, DriverError, DriverInfo, Endpoint, Guard,
    HidChannel, Identification, Lighting, LightingTraits, Maturity, Note, Outcome, Rgb, Rollback,
    Snapshot,
};
use std::sync::Arc;
use std::time::Duration;

const INFO: DriverInfo = DriverInfo {
    id: ID,
    name: "BY Tech (BYCOMBO4)",
    maturity: Maturity::Experimental,
    requirement: None,
};

/// The vendor waits 20 ms between a request and its Get Feature, and 30 ms between commands.
const REPLY_DELAY: Duration = Duration::from_millis(20);
const COMMAND_GAP: Duration = Duration::from_millis(30);
/// Time for the keyboard to store a block before it is read back.
const STORE_DELAY: Duration = Duration::from_millis(200);

pub struct ByTechDriver;

impl Driver for ByTechDriver {
    fn info(&self) -> DriverInfo {
        INFO
    }

    fn admits(&self, endpoint: &Endpoint) -> bool {
        admits(endpoint)
    }

    fn discover(&self, bus: &Arc<dyn Bus>) -> Discovery {
        let mut found = Discovery::default();
        for endpoint in bus.hid_endpoints().iter().filter(|e| admits(e)) {
            match ByTechDevice::confirm(Arc::clone(bus), endpoint.clone()) {
                Ok(device) => found.devices.push(Box::new(device)),
                Err(error) => found.notes.push(Note {
                    subject: Some(endpoint.product.clone()),
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
            let _lease = bus.lock(&lock_name(endpoint))?;
            let channel = bus.open_hid(endpoint)?;
            let password = read(channel.as_ref(), Command::ReadPassword)?;
            let known = PASSWORDS.iter().any(|p| *p == password[..]);
            let mut payload = password.clone();
            let mut details = serde_json::json!({
                "password": codec::hex(&password),
                "password_known": known,
            });
            // The configuration is read only from a keyboard that proved its family.
            if known {
                std::thread::sleep(COMMAND_GAP);
                let raw = read(channel.as_ref(), Command::ReadConfig)?;
                details["config_valid"] = Config::parse(&raw).is_ok().into();
                details["effect"] = raw[0x0a].into();
                payload.extend_from_slice(&raw);
            }
            Ok(Identification {
                driver: ID.into(),
                firmware_version: None,
                reported_vid: None,
                reported_pid: None,
                details,
                payload_hex: codec::hex(&payload),
            })
        })
    }
}

fn lock_name(endpoint: &Endpoint) -> String {
    format!(
        "Local\\{}-{}",
        identity::LOCK_PREFIX,
        &endpoint.fingerprint()[..16]
    )
}

/// A read request, then its answer. A failed read changes nothing and may be repeated by the
/// user; this function itself never repeats.
fn read(channel: &dyn HidChannel, command: Command) -> Result<Vec<u8>, DriverError> {
    channel.send_feature(&codec::request(command)?)?;
    std::thread::sleep(REPLY_DELAY);
    let mut report = [0; REPORT_BYTES];
    report[0] = REPORT_ID;
    let n = channel.get_feature(&mut report)?;
    Ok(codec::reply(&report[..n], command)?.to_vec())
}

/// A write is sent once: if the transfer fails, the packet may still have arrived.
fn write(channel: &dyn HidChannel, command: Command, data: &[u8]) -> Result<(), DriverError> {
    let packet = codec::write(command, data)?;
    channel
        .send_feature(&packet)
        .map_err(|e| DriverError::Unconfirmed(e.to_string()))
}

fn read_blocks(channel: &dyn HidChannel) -> Result<(Config, ColorTable), DriverError> {
    let config = Config::parse(&read(channel, Command::ReadConfig)?)?;
    std::thread::sleep(COMMAND_GAP);
    let colors = ColorTable::parse(&read(channel, Command::ReadColors)?)?;
    Ok((config, colors))
}

/// One operation's hold on the device; the channel closes before the lock is released.
struct Exchange {
    channel: Box<dyn HidChannel>,
    _lease: Box<dyn Guard>,
}

pub(super) struct ByTechDevice {
    bus: Arc<dyn Bus>,
    endpoint: Endpoint,
    descriptor: DeviceDescriptor,
    identity: String,
    protocol_identity: String,
    lock: String,
    original: Option<(Config, ColorTable)>,
}

impl ByTechDevice {
    /// Confirms the family by the password and checks both blocks parse. Sends no write.
    fn confirm(bus: Arc<dyn Bus>, endpoint: Endpoint) -> Result<Self, DriverError> {
        let lock = lock_name(&endpoint);
        let _lease = bus.lock(&lock)?;
        let channel = bus.open_hid(&endpoint)?;
        let protocol_identity = Self::password(channel.as_ref(), &endpoint)?;
        std::thread::sleep(COMMAND_GAP);
        read_blocks(channel.as_ref())?;
        drop(channel);
        Ok(Self {
            identity: endpoint.fingerprint(),
            descriptor: descriptor(&endpoint, INFO),
            bus,
            endpoint,
            protocol_identity,
            lock,
            original: None,
        })
    }

    /// The vendor's own family check: the six password bytes must equal a `Psd` from the package.
    fn password(channel: &dyn HidChannel, endpoint: &Endpoint) -> Result<String, DriverError> {
        let password = read(channel, Command::ReadPassword)?;
        if !PASSWORDS.iter().any(|p| *p == password[..]) {
            return Err(DriverError::rejected(format!(
                "password {} is not an AK820 MAX one",
                codec::hex(&password)
            )));
        }
        Ok(format!(
            "{}-{:04x}",
            codec::hex(&password),
            endpoint.release
        ))
    }

    fn exchange(&self) -> Result<Exchange, DriverError> {
        let lease = self.bus.lock(&self.lock)?;
        let channel = self.bus.open_hid(&self.endpoint)?;
        if Self::password(channel.as_ref(), &self.endpoint)? != self.protocol_identity {
            return Err(DriverError::Changed);
        }
        std::thread::sleep(COMMAND_GAP);
        Ok(Exchange {
            channel,
            _lease: lease,
        })
    }

    /// Writes only the blocks that differ from `current`, then reads both back.
    fn write_blocks(
        channel: &dyn HidChannel,
        current: &(Config, ColorTable),
        target: &(Config, ColorTable),
    ) -> Result<Outcome, DriverError> {
        if current.0 != target.0 {
            write(channel, Command::WriteConfig, target.0.bytes())?;
            std::thread::sleep(COMMAND_GAP);
        }
        if current.1 != target.1 {
            write(channel, Command::WriteColors, target.1.bytes())?;
        }
        std::thread::sleep(STORE_DELAY);
        if read_blocks(channel)? == *target {
            Ok(Outcome::Confirmed)
        } else {
            Err(DriverError::ReadBackMismatch)
        }
    }

    fn restore_to(&self, target: &(Config, ColorTable)) -> Result<Outcome, DriverError> {
        let exchange = self.exchange()?;
        let channel = exchange.channel.as_ref();
        let current = read_blocks(channel)?;
        std::thread::sleep(COMMAND_GAP);
        Self::write_blocks(channel, &current, target).map(|outcome| match outcome {
            Outcome::Confirmed => Outcome::Restored,
            other => other,
        })
    }
}

impl Device for ByTechDevice {
    fn descriptor(&self) -> &DeviceDescriptor {
        &self.descriptor
    }

    fn lighting(&mut self) -> Option<&mut dyn Lighting> {
        Some(self)
    }
}

impl Lighting for ByTechDevice {
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
        let current = read_blocks(channel)?;
        let target = (
            current.0.solid(),
            current.1.with_static_color(color.to_array()),
        );
        if self.original.is_none() {
            let backup = Backup::new(
                &self.identity,
                &self.protocol_identity,
                &current.0,
                &current.1,
            );
            self.bus
                .store_backup("by-tech", &self.identity, &backup.to_json()?)?;
            self.original = Some(current.clone());
        }
        std::thread::sleep(COMMAND_GAP);
        Self::write_blocks(channel, &current, &target)
    }

    fn restore(&mut self) -> Result<Outcome, DriverError> {
        let original = self.original.clone().ok_or(DriverError::NoRollback)?;
        self.restore_to(&original)
    }

    fn restore_snapshot(&mut self, snapshot: &Snapshot) -> Result<Outcome, DriverError> {
        let backup: Backup = snapshot.parse()?;
        let target = backup.blocks(&self.identity, &self.protocol_identity)?;
        self.restore_to(&target)
    }
}
