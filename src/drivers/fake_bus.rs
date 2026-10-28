//! A scripted bus for driver tests: records every packet and answers through a closure, so the
//! exchange order of a driver can be checked without hardware.

use crate::core::{Bus, DriverError, Endpoint, Guard, HidChannel, Stream};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// What the device does with a packet: the input reports it queues in reply.
pub type Responder = Box<dyn FnMut(&Packet) -> Vec<Vec<u8>> + Send>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Packet {
    Output(Vec<u8>),
    Feature(Vec<u8>),
}

struct State {
    log: Vec<Packet>,
    input: VecDeque<Vec<u8>>,
    feature: VecDeque<Vec<u8>>,
    responder: Responder,
    backups: Vec<(String, Vec<u8>, usize)>,
}

pub struct FakeBus {
    endpoints: Vec<Endpoint>,
    state: Arc<Mutex<State>>,
}

impl FakeBus {
    pub fn new(endpoints: Vec<Endpoint>, responder: Responder) -> Arc<Self> {
        Arc::new(Self {
            endpoints,
            state: Arc::new(Mutex::new(State {
                log: Vec::new(),
                input: VecDeque::new(),
                feature: VecDeque::new(),
                responder,
                backups: Vec::new(),
            })),
        })
    }

    pub fn log(&self) -> Vec<Packet> {
        self.state.lock().map(|s| s.log.clone()).unwrap_or_default()
    }

    pub fn backups(&self) -> Vec<(String, Vec<u8>)> {
        self.state
            .lock()
            .map(|s| {
                s.backups
                    .iter()
                    .map(|(k, j, _)| (k.clone(), j.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The number of packets sent before the first backup was stored.
    pub fn packets_before_backup(&self) -> Option<usize> {
        self.state
            .lock()
            .ok()
            .and_then(|s| s.backups.first().map(|(_, _, at)| *at))
    }
}

struct Channel(Arc<Mutex<State>>);

impl Channel {
    fn record(&self, packet: Packet, to_feature: bool) {
        if let Ok(mut s) = self.0.lock() {
            let replies = (s.responder)(&packet);
            s.log.push(packet);
            let queue = if to_feature {
                &mut s.feature
            } else {
                &mut s.input
            };
            queue.extend(replies);
        }
    }
}

fn pop_into(queue: &mut VecDeque<Vec<u8>>, buffer: &mut [u8]) -> usize {
    queue.pop_front().map_or(0, |data| {
        let n = data.len().min(buffer.len());
        buffer[..n].copy_from_slice(&data[..n]);
        n
    })
}

impl HidChannel for Channel {
    fn write(&self, data: &[u8]) -> Result<usize, DriverError> {
        self.record(Packet::Output(data.to_vec()), false);
        Ok(data.len())
    }

    fn read_timeout(&self, buffer: &mut [u8], _milliseconds: i32) -> Result<usize, DriverError> {
        Ok(self
            .0
            .lock()
            .map(|mut s| pop_into(&mut s.input, buffer))
            .unwrap_or(0))
    }

    fn send_feature(&self, data: &[u8]) -> Result<(), DriverError> {
        self.record(Packet::Feature(data.to_vec()), true);
        Ok(())
    }

    fn get_feature(&self, buffer: &mut [u8]) -> Result<usize, DriverError> {
        Ok(self
            .0
            .lock()
            .map(|mut s| pop_into(&mut s.feature, buffer))
            .unwrap_or(0))
    }

    fn report_descriptor(&self, _buffer: &mut [u8]) -> Result<usize, DriverError> {
        Err(DriverError::Io("fake bus has no descriptors".into()))
    }

    fn container_id(&self) -> Result<u128, DriverError> {
        Ok(1)
    }
}

struct Unlocked;
impl Guard for Unlocked {}

impl Bus for FakeBus {
    fn hid_endpoints(&self) -> &[Endpoint] {
        &self.endpoints
    }

    fn open_hid(&self, _endpoint: &Endpoint) -> Result<Box<dyn HidChannel>, DriverError> {
        Ok(Box::new(Channel(Arc::clone(&self.state))))
    }

    fn connect_loopback(&self, port: u16) -> Result<Box<dyn Stream>, DriverError> {
        let stream = std::net::TcpStream::connect(("127.0.0.1", port))?;
        Ok(Box::new(stream))
    }

    fn lock(&self, _name: &str) -> Result<Box<dyn Guard>, DriverError> {
        Ok(Box::new(Unlocked))
    }

    fn store_backup(&self, kind: &str, _device: &str, json: &[u8]) -> Result<(), DriverError> {
        if let Ok(mut s) = self.state.lock() {
            let at = s.log.len();
            s.backups.push((kind.into(), json.to_vec(), at));
        }
        Ok(())
    }
}
