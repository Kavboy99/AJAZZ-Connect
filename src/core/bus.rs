use super::{DriverError, Endpoint};
use std::io::{Read, Write};

/// A held lock. Releasing happens on drop, on the thread that took it.
pub trait Guard {}

/// An opened HID collection. It lives for one operation and closes on drop.
pub trait HidChannel {
    /// Output report; the first byte is the Report ID. Returns the byte count the OS reported,
    /// which may be zero after a synchronous completion.
    fn write(&self, data: &[u8]) -> Result<usize, DriverError>;
    fn read_timeout(&self, buffer: &mut [u8], milliseconds: i32) -> Result<usize, DriverError>;
    fn send_feature(&self, data: &[u8]) -> Result<(), DriverError>;
    fn get_feature(&self, buffer: &mut [u8]) -> Result<usize, DriverError>;
    fn report_descriptor(&self, buffer: &mut [u8]) -> Result<usize, DriverError>;
    /// Identifier of the physical USB device the collection belongs to.
    fn container_id(&self) -> Result<u128, DriverError>;
}

/// A byte stream to a local service.
pub trait Stream: Read + Write + Send {}

impl<T: Read + Write + Send> Stream for T {}

/// The only door from a driver to the outside world. Drivers never touch the OS directly, so
/// every exchange can be replayed against a fake bus in tests.
pub trait Bus: Send + Sync {
    /// HID collections found by the last enumeration.
    fn hid_endpoints(&self) -> &[Endpoint];
    fn open_hid(&self, endpoint: &Endpoint) -> Result<Box<dyn HidChannel>, DriverError>;
    /// TCP connection to `127.0.0.1:port`; nothing else is reachable.
    fn connect_loopback(&self, port: u16) -> Result<Box<dyn Stream>, DriverError>;
    /// A named system-wide lock shared with other programs that drive the same device.
    fn lock(&self, name: &str) -> Result<Box<dyn Guard>, DriverError>;
    /// Persists a rollback file written before the first change of a device.
    fn store_backup(&self, kind: &str, device: &str, json: &[u8]) -> Result<(), DriverError>;
}
