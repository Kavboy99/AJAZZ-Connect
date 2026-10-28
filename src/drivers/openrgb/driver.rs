use super::ID;
use super::backup::Backup;
use super::client::Client;
use super::wire::{self, Cursor, SdkDevice};
use crate::core::{
    Bus, Detail, Device, DeviceDescriptor, Discovery, Driver, DriverError, DriverInfo, Endpoint,
    Identification, Lighting, LightingTraits, Maturity, Note, Outcome, Requirement, Rgb, Rollback,
    Snapshot,
};
use std::sync::{Arc, Mutex, MutexGuard};

const DEFAULT_PORT: u16 = 6742;

const INFO: DriverInfo = DriverInfo {
    id: ID,
    name: "OpenRGB SDK",
    maturity: Maturity::Stable,
    requirement: Some(Requirement::LocalService {
        name: "OpenRGB",
        address: "127.0.0.1:6742",
    }),
};

pub struct OpenRgbDriver {
    port: u16,
}

impl OpenRgbDriver {
    pub fn new() -> Self {
        Self { port: DEFAULT_PORT }
    }

    #[cfg(test)]
    pub(super) fn on_port(port: u16) -> Self {
        Self { port }
    }

    fn connect(&self, bus: &Arc<dyn Bus>) -> Result<Vec<Box<dyn Device>>, DriverError> {
        let mut client = Client::handshake(bus.connect_loopback(self.port)?)?;
        let sdk: Vec<SdkDevice> = (0..client.count()?)
            .map(|index| client.device(index))
            .collect::<Result<_, _>>()?;
        let client = Arc::new(Mutex::new(client));
        Ok(sdk
            .into_iter()
            .map(|sdk| {
                Box::new(OpenRgbDevice::new(
                    Arc::clone(bus),
                    Arc::clone(&client),
                    sdk,
                )) as Box<dyn Device>
            })
            .collect())
    }
}

impl Default for OpenRgbDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl Driver for OpenRgbDriver {
    fn info(&self) -> DriverInfo {
        INFO
    }

    /// The SDK server owns its devices; this driver never claims a HID endpoint.
    fn admits(&self, _: &Endpoint) -> bool {
        false
    }

    fn discover(&self, bus: &Arc<dyn Bus>) -> Discovery {
        match self.connect(bus) {
            Ok(devices) => Discovery {
                devices,
                notes: Vec::new(),
            },
            Err(error) => Discovery {
                devices: Vec::new(),
                notes: vec![Note {
                    subject: None,
                    error,
                }],
            },
        }
    }

    fn identify(&self, _: &dyn Bus, _: &Endpoint) -> Option<Result<Identification, DriverError>> {
        None
    }
}

pub(super) struct OpenRgbDevice {
    bus: Arc<dyn Bus>,
    client: Arc<Mutex<Client>>,
    sdk: SdkDevice,
    descriptor: DeviceDescriptor,
    backup: Option<Backup>,
}

impl OpenRgbDevice {
    fn new(bus: Arc<dyn Bus>, client: Arc<Mutex<Client>>, sdk: SdkDevice) -> Self {
        let descriptor = DeviceDescriptor {
            kind: sdk.device_kind(),
            vendor: (!sdk.vendor.trim().is_empty()).then(|| sdk.vendor.trim().to_owned()),
            name: sdk.name.clone(),
            driver: INFO,
            details: vec![Detail::Leds(sdk.led_count)],
            visual: None,
        };
        Self {
            bus,
            client,
            sdk,
            descriptor,
            backup: None,
        }
    }

    fn client(&self) -> Result<MutexGuard<'_, Client>, DriverError> {
        self.client
            .lock()
            .map_err(|_| DriverError::Io("SDK client lock poisoned".into()))
    }

    /// The current server state of this device, refused if it can no longer take Direct colors.
    fn fresh(client: &mut Client, sdk: &SdkDevice) -> Result<SdkDevice, DriverError> {
        let fresh = client.refresh(sdk)?;
        if !fresh.writable() {
            return Err(DriverError::rejected(
                "no Direct mode without automatic saving",
            ));
        }
        Ok(fresh)
    }

    fn replay(&self, backup: &Backup) -> Result<Outcome, DriverError> {
        let mut client = self.client()?;
        let version = client.version;
        let d = Self::fresh(&mut client, &self.sdk)?;
        backup.check(&d, version)?;
        // Accept only the exact server-defined mode; runtime parameters may differ.
        let mut c = Cursor::new(&backup.mode_wire);
        let mode = wire::parse_mode(&mut c, version)?;
        if !c.at_end() || !mode.same_definition(&d.modes[backup.mode_index as usize]) {
            return Err(DriverError::ForeignBackup);
        }
        // Colors go first while Direct is still active, then the hardware effect comes back.
        client.set_colors(d.index, &backup.colors)?;
        client.set_mode(d.index, backup.mode_index, &backup.mode_wire)?;
        let after = client.device(d.index)?;
        if after.identity == d.identity
            && after.active == backup.mode_index
            && after.colors == backup.colors
        {
            Ok(Outcome::Restored)
        } else {
            Err(DriverError::ReadBackMismatch)
        }
    }
}

impl Device for OpenRgbDevice {
    fn descriptor(&self) -> &DeviceDescriptor {
        &self.descriptor
    }

    fn lighting(&mut self) -> Option<&mut dyn Lighting> {
        Some(self)
    }
}

impl Lighting for OpenRgbDevice {
    fn traits(&self) -> LightingTraits {
        LightingTraits {
            writable: self.sdk.writable(),
            persistent: false,
            rollback: Rollback::SessionAndFile,
        }
    }

    fn apply_solid(&mut self, color: Rgb) -> Result<Outcome, DriverError> {
        let client = Arc::clone(&self.client);
        let mut client = client
            .lock()
            .map_err(|_| DriverError::Io("SDK client lock poisoned".into()))?;
        let d = Self::fresh(&mut client, &self.sdk)?;
        if self.backup.is_none() {
            let backup = Backup::of(&d, client.version)
                .ok_or_else(|| DriverError::protocol("no active mode to back up"))?;
            self.bus
                .store_backup("openrgb", &d.identity, &backup.to_json()?)?;
            self.backup = Some(backup);
        }
        let direct = d
            .direct()
            .ok_or_else(|| DriverError::rejected("no Direct mode"))?;
        let mut mode = d.modes[direct].wire.clone();
        let at = d.modes[direct].color_mode_at;
        mode[at..at + 4].copy_from_slice(&1u32.to_le_bytes());
        client.set_mode(d.index, direct as u32, &mode)?;
        let colors = vec![[color.r, color.g, color.b, 0]; d.led_count];
        client.set_colors(d.index, &colors)?;
        let after = client.device(d.index)?;
        if after.identity == d.identity && after.active == direct as u32 && after.colors == colors {
            Ok(Outcome::ServiceConfirmed)
        } else {
            Err(DriverError::ReadBackMismatch)
        }
    }

    fn restore(&mut self) -> Result<Outcome, DriverError> {
        let backup = self.backup.clone().ok_or(DriverError::NoRollback)?;
        self.replay(&backup)
    }

    fn restore_snapshot(&mut self, snapshot: &Snapshot) -> Result<Outcome, DriverError> {
        self.replay(&snapshot.parse()?)
    }
}
