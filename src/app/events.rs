use crate::core::{Device, DeviceDescriptor, DriverError, DriverInfo, Outcome};
use crate::diagnostics::Report;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

/// Wakes the interface after a worker finished; the interface repaints only then.
pub type Waker = Arc<dyn Fn() + Send + Sync>;

/// Why a driver (or HID access as a whole, when `driver` is `None`) found nothing.
#[derive(Clone, Debug)]
pub struct DriverNote {
    pub driver: Option<DriverInfo>,
    pub subject: Option<String>,
    pub error: DriverError,
}

pub struct Discovered {
    pub devices: Vec<Box<dyn Device>>,
    /// Devices whose driver is experimental and not enabled: shown, but never contacted.
    pub dormant: Vec<DeviceDescriptor>,
    pub notes: Vec<DriverNote>,
    /// Every endpoint a driver admits (`driver id / port fingerprint`), to tell a later automatic
    /// search whether anything relevant changed.
    pub admitted: Vec<String>,
}

pub enum Event {
    Battery(Vec<BatteryResult>),
    Discovered(Discovered),
    /// An automatic search found the same interfaces as before; nothing was sent to any device.
    Unchanged,
    Lighting {
        slot: usize,
        device: Box<dyn Device>,
        result: Result<Outcome, DriverError>,
    },
    /// Boxed: a report is far larger than the other events.
    Probed(Box<Result<Report, DriverError>>),
}

pub struct BatteryResult {
    pub slot: usize,
    pub device: Box<dyn Device>,
    pub result: Result<(), DriverError>,
}

/// Each task gets its own thread for the duration of one operation; no pool stays alive.
pub struct Events {
    tx: Sender<Event>,
    rx: Receiver<Event>,
    waker: Waker,
}

impl Events {
    pub fn new(waker: Waker) -> Self {
        let (tx, rx) = channel();
        Self { tx, rx, waker }
    }

    pub fn spawn(&self, job: impl FnOnce() -> Event + Send + 'static) {
        let tx = self.tx.clone();
        let waker = Arc::clone(&self.waker);
        std::thread::spawn(move || {
            // The receiver is gone only when the application is closing.
            let _ = tx.send(job());
            waker();
        });
    }

    pub fn try_next(&self) -> Option<Event> {
        self.rx.try_recv().ok()
    }
}
