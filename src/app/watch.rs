//! Device arrival and removal, turned into one automatic search. Windows reports every HID
//! collection of a device separately (a keyboard has eight), so notifications are gathered until
//! the system settles, and only then is the search started.

use crate::transport::DeviceWatch;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// How long the device list must stay quiet before an automatic search starts.
const SETTLE: Duration = Duration::from_millis(400);

/// Asks the interface for one frame after the given delay. No timer of our own runs.
pub type WakeAfter = Arc<dyn Fn(Duration) + Send + Sync>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Due {
    Nothing,
    Now,
    After(Duration),
}

pub struct Watch {
    changed: Arc<Mutex<Option<Instant>>>,
    _os: Option<DeviceWatch>,
}

impl Watch {
    /// Starts listening; the first search is due right away.
    pub fn start(wake_after: WakeAfter) -> Self {
        let start = Instant::now()
            .checked_sub(SETTLE)
            .unwrap_or_else(Instant::now);
        let changed = Arc::new(Mutex::new(Some(start)));
        let flag = Arc::clone(&changed);
        let os = DeviceWatch::start(Box::new(move || {
            if let Ok(mut at) = flag.lock() {
                *at = Some(Instant::now());
            }
            wake_after(SETTLE);
        }));
        Self { changed, _os: os }
    }

    pub fn due(&self) -> Due {
        let Ok(at) = self.changed.lock() else {
            return Due::Nothing;
        };
        match *at {
            None => Due::Nothing,
            Some(at) => match SETTLE.checked_sub(at.elapsed()) {
                Some(left) if !left.is_zero() => Due::After(left),
                _ => Due::Now,
            },
        }
    }

    pub fn clear(&self) {
        if let Ok(mut at) = self.changed.lock() {
            *at = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_search_is_due_at_once_and_bursts_wait_to_settle() {
        let watch = Watch::start(Arc::new(|_| {}));
        // The registration itself must succeed on Windows: it checks the FFI filter layout.
        assert_eq!(watch._os.is_some(), cfg!(windows));
        assert_eq!(watch.due(), Due::Now);
        watch.clear();
        assert_eq!(watch.due(), Due::Nothing);
        if let Ok(mut at) = watch.changed.lock() {
            *at = Some(Instant::now());
        }
        assert!(matches!(watch.due(), Due::After(left) if left <= SETTLE));
    }
}
