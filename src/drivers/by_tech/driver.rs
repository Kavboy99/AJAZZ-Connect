mod commit;
mod configuration;
mod controls;

use super::codec::{self, Command, PASSWORDS, REPORT_BYTES, REPORT_ID};
use super::state::{ColorTable, Config};
use super::{ID, admits, descriptor};
use crate::app::identity;
use crate::core::{
    Bus, Device, DeviceDescriptor, Discovery, Driver, DriverError, DriverInfo, Endpoint, Guard,
    HidChannel, Identification, Lighting, Maturity, Note,
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

pub struct ByTechDriver;

impl Driver for ByTechDriver {
    fn supports_configuration_backup(&self) -> bool {
        true
    }
    fn info(&self) -> DriverInfo {
        INFO
    }

    fn lighting_preview(&self) -> Option<crate::core::LightingPreview> {
        Some(super::lighting::preview())
    }

    fn admits(&self, endpoint: &Endpoint) -> bool {
        admits(endpoint) || super::radio::admits(endpoint)
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
        // A passive USB match must not hide a usable receiver when wired confirmation fails.
        if found.devices.is_empty() {
            super::radio::discover(bus, INFO, &mut found);
        }
        found
    }

    fn identify(
        &self,
        bus: &dyn Bus,
        endpoint: &Endpoint,
    ) -> Option<Result<Identification, DriverError>> {
        if super::radio::admits(endpoint) {
            // The passive probe records its descriptor; discovery performs the radio battery read.
            return None;
        }
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
                std::thread::sleep(COMMAND_GAP);
                let palette = read(channel.as_ref(), Command::ReadColors)?;
                ColorTable::parse(&palette)?;
                details["lighting_programs"] = serde_json::Value::Array((1usize..=21).map(|id| {
                    let at = 0x38 + id * 2;
                    let colors: Vec<_> = (0..7).map(|slot| {
                        let rgb = 21 * id + 3 * slot;
                        serde_json::json!(&palette[rgb..rgb + 3])
                    }).collect();
                    serde_json::json!({"hardware_id":id, "speed":raw[at+1] >> 4, "brightness":raw[at], "color_slot":raw[at+1] & 15, "colors":colors})
                }).collect());
                payload.extend_from_slice(&raw);
                if password == PASSWORDS[1] && endpoint.release == 0x0100 {
                    super::battery::identify(channel.as_ref(), &mut details);
                }
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

pub(super) fn lock_name(endpoint: &Endpoint) -> String {
    format!(
        "Local\\{}-{}",
        identity::LOCK_PREFIX,
        &endpoint.fingerprint()[..16]
    )
}

/// A read request, then its answer. A failed read changes nothing and may be repeated by the
/// user; this function itself never repeats.
pub(super) fn read(channel: &dyn HidChannel, command: Command) -> Result<Vec<u8>, DriverError> {
    let request = codec::request(command)?;
    crate::diagnostics::logging::command(ID, request[1], false);
    channel.send_feature(&request)?;
    std::thread::sleep(REPLY_DELAY);
    let mut report = [0; REPORT_BYTES];
    report[0] = REPORT_ID;
    let n = channel.get_feature(&mut report)?;
    let framed = report
        .get(..n)
        .ok_or_else(|| DriverError::protocol("oversized Feature reply"))?;
    Ok(codec::reply(framed, command)?.to_vec())
}

/// One operation's hold on the device; the channel closes before the lock is released.
pub(super) struct Exchange {
    pub(super) channel: Box<dyn HidChannel>,
    _lease: Box<dyn Guard>,
}

pub(super) struct ByTechDevice {
    bus: Arc<dyn Bus>,
    endpoint: Endpoint,
    pub(super) descriptor: DeviceDescriptor,
    identity: String,
    protocol_identity: String,
    lock: String,

    cached: Option<Config>,
    cached_colors: Option<ColorTable>,
}

impl ByTechDevice {
    /// Confirms the family by the password and checks both blocks parse. Sends no write.
    fn confirm(bus: Arc<dyn Bus>, endpoint: Endpoint) -> Result<Self, DriverError> {
        let lock = lock_name(&endpoint);
        let _lease = bus.lock(&lock)?;
        let channel = bus.open_hid(&endpoint)?;
        let protocol_identity = Self::password(channel.as_ref(), &endpoint)?;
        std::thread::sleep(COMMAND_GAP);
        let cached = Config::parse(&read(channel.as_ref(), Command::ReadConfig)?).ok();
        std::thread::sleep(COMMAND_GAP);
        let cached_colors = ColorTable::parse(&read(channel.as_ref(), Command::ReadColors)?).ok();
        let mut descriptor = descriptor(&endpoint, INFO);
        if protocol_identity == "03000000015e-0100" {
            super::battery::append(channel.as_ref(), &mut descriptor);
        }
        drop(channel);
        Ok(Self {
            identity: endpoint.fingerprint(),
            descriptor,
            bus,
            endpoint,
            protocol_identity,
            lock,

            cached,
            cached_colors,
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

    pub(super) fn exchange(&self) -> Result<Exchange, DriverError> {
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
}

impl Device for ByTechDevice {
    fn configuration_backup(&mut self) -> Option<&mut dyn crate::core::ConfigurationBackup> {
        Some(self)
    }
    fn battery(&mut self) -> Option<&mut dyn crate::core::Battery> {
        (self.protocol_identity == "03000000015e-0100").then_some(self)
    }
    fn descriptor(&self) -> &DeviceDescriptor {
        &self.descriptor
    }

    fn lighting(&mut self) -> Option<&mut dyn Lighting> {
        Some(self)
    }
}
