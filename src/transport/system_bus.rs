use super::{lease, tcp};
use crate::core::{Bus, DriverError, Endpoint, Guard, HidChannel, Stream};
use crate::storage::backups;

/// The real bus: one HID enumeration per instance. Created for a discovery or a probe and dropped
/// with the devices it served, so nothing stays open in the background.
pub struct SystemBus {
    #[cfg(windows)]
    hid: Option<std::sync::Mutex<super::hid::HidHost>>,
    endpoints: Vec<Endpoint>,
    hid_error: Option<DriverError>,
}

impl SystemBus {
    #[cfg(windows)]
    pub fn new() -> Self {
        match super::hid::HidHost::enumerate() {
            Ok((host, endpoints)) => Self {
                hid: Some(std::sync::Mutex::new(host)),
                endpoints,
                hid_error: None,
            },
            Err(error) => Self {
                hid: None,
                endpoints: Vec::new(),
                hid_error: Some(error),
            },
        }
    }

    #[cfg(not(windows))]
    pub fn new() -> Self {
        Self {
            endpoints: Vec::new(),
            hid_error: Some(DriverError::Unavailable(
                "HID access is implemented for Windows".into(),
            )),
        }
    }

    /// Why HID enumeration failed, if it did. Loopback services still work in that case.
    pub fn hid_error(&self) -> Option<&DriverError> {
        self.hid_error.as_ref()
    }
}

impl Default for SystemBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Bus for SystemBus {
    fn hid_endpoints(&self) -> &[Endpoint] {
        &self.endpoints
    }

    #[cfg(windows)]
    fn open_hid(&self, endpoint: &Endpoint) -> Result<Box<dyn HidChannel>, DriverError> {
        let host = self
            .hid
            .as_ref()
            .ok_or_else(|| DriverError::Unavailable("HID enumeration failed".into()))?;
        let host = host
            .lock()
            .map_err(|_| DriverError::Io("HID host lock poisoned".into()))?;
        host.open(endpoint)
    }

    #[cfg(not(windows))]
    fn open_hid(&self, _endpoint: &Endpoint) -> Result<Box<dyn HidChannel>, DriverError> {
        Err(DriverError::Unavailable(
            "HID access is implemented for Windows".into(),
        ))
    }

    fn connect_loopback(&self, port: u16) -> Result<Box<dyn Stream>, DriverError> {
        Ok(Box::new(tcp::connect_loopback(port)?))
    }

    fn lock(&self, name: &str) -> Result<Box<dyn Guard>, DriverError> {
        lease::acquire(name)
    }

    fn store_backup(&self, kind: &str, device: &str, json: &[u8]) -> Result<(), DriverError> {
        backups::store(kind, device, json).map_err(DriverError::from)
    }
}
