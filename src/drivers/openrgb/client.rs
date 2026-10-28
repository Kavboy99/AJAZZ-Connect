//! One SDK connection shared by every device the server reports.

use super::wire::{self, SdkDevice};
use crate::app::identity;
use crate::core::{DriverError, Stream};

pub struct Client {
    stream: Box<dyn Stream>,
    pub version: u32,
}

impl Client {
    /// Negotiates the protocol level and introduces the application. No device is touched.
    pub fn handshake(stream: Box<dyn Stream>) -> Result<Self, DriverError> {
        let mut client = Self { stream, version: 3 };
        client.send(0, wire::PROTOCOL_VERSION, &3u32.to_le_bytes())?;
        let reply = client.receive(0, wire::PROTOCOL_VERSION)?;
        let server = match reply.as_slice() {
            [a, b, c, d] => u32::from_le_bytes([*a, *b, *c, *d]),
            _ => return Err(DriverError::protocol("SDK version reply")),
        };
        if server < 1 {
            return Err(DriverError::Unavailable("OpenRGB SDK protocol 0".into()));
        }
        client.version = server.min(3);
        let mut name = identity::CLIENT_NAME.as_bytes().to_vec();
        name.push(0);
        client.send(0, wire::SET_CLIENT_NAME, &name)?;
        Ok(client)
    }

    fn send(&mut self, device: u32, packet: u32, payload: &[u8]) -> Result<(), DriverError> {
        self.stream
            .write_all(&wire::frame(device, packet, payload)?)?;
        Ok(())
    }

    fn receive(&mut self, device: u32, packet: u32) -> Result<Vec<u8>, DriverError> {
        let mut h = [0u8; 16];
        self.stream.read_exact(&mut h)?;
        let (id, kind, n) = wire::header(&h)?;
        if kind == wire::DEVICE_LIST_UPDATED {
            // Indexes may now point at other devices: stop instead of guessing.
            return Err(DriverError::Changed);
        }
        if id != device || kind != packet {
            return Err(DriverError::protocol(format!(
                "unexpected SDK packet {id}:{kind}"
            )));
        }
        let mut payload = vec![0; n];
        self.stream.read_exact(&mut payload)?;
        Ok(payload)
    }

    pub fn count(&mut self) -> Result<u32, DriverError> {
        self.send(0, wire::REQUEST_COUNT, &[])?;
        let data = self.receive(0, wire::REQUEST_COUNT)?;
        let n = match data.as_slice() {
            [a, b, c, d] => u32::from_le_bytes([*a, *b, *c, *d]),
            _ => return Err(DriverError::protocol("SDK count reply")),
        };
        if n > 128 {
            return Err(DriverError::protocol("too many SDK devices"));
        }
        Ok(n)
    }

    pub fn device(&mut self, index: u32) -> Result<SdkDevice, DriverError> {
        let version = self.version;
        self.send(index, wire::REQUEST_DEVICE, &version.to_le_bytes())?;
        wire::parse_device(&self.receive(index, wire::REQUEST_DEVICE)?, version, index)
    }

    /// The device at the remembered index, refreshed, after checking it is still the same one.
    pub fn refresh(&mut self, known: &SdkDevice) -> Result<SdkDevice, DriverError> {
        if known.index >= self.count()? {
            return Err(DriverError::Changed);
        }
        let fresh = self.device(known.index)?;
        if fresh.identity != known.identity {
            return Err(DriverError::Changed);
        }
        Ok(fresh)
    }

    pub fn set_mode(&mut self, index: u32, mode: u32, wire_mode: &[u8]) -> Result<(), DriverError> {
        self.send(
            index,
            wire::UPDATE_MODE,
            &wire::mode_payload(mode, wire_mode),
        )
    }

    pub fn set_colors(&mut self, index: u32, colors: &[[u8; 4]]) -> Result<(), DriverError> {
        self.send(index, wire::UPDATE_LEDS, &wire::colors_payload(colors)?)
    }
}
