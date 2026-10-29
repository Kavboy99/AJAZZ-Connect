use super::*;
use crate::app::events::Event;
use crate::core::*;
use std::sync::{Arc, Mutex};
struct Keyboard {
    descriptor: DeviceDescriptor,
    calls: Arc<Mutex<Vec<u8>>>,
    persistent: bool,
}
impl Device for Keyboard {
    fn descriptor(&self) -> &DeviceDescriptor {
        &self.descriptor
    }
    fn lighting(&mut self) -> Option<&mut dyn Lighting> {
        Some(self)
    }
}
impl Lighting for Keyboard {
    fn restore_snapshot(&mut self, _: &Snapshot) -> Result<Outcome, DriverError> {
        Err(DriverError::NoRollback)
    }
    fn traits(&self) -> LightingTraits {
        LightingTraits {
            writable: true,
            persistent: self.persistent,
            rollback: Rollback::SessionAndFile,
        }
    }
    fn editor(&self) -> Option<LightingEditor> {
        Some(LightingEditor {
            current: 1,
            presets: vec![],
        })
    }
    fn apply_settings(&mut self, settings: LightingSettings) -> Result<Outcome, DriverError> {
        self.calls.lock().unwrap().push(settings.speed);
        if settings.speed == 3 {
            Err(DriverError::ReadBackMismatch)
        } else {
            Ok(Outcome::Confirmed)
        }
    }
    fn apply_solid(&mut self, _: Rgb) -> Result<Outcome, DriverError> {
        Err(DriverError::NoRollback)
    }
    fn restore(&mut self) -> Result<Outcome, DriverError> {
        Ok(Outcome::Restored)
    }
}
fn settings(speed: u8) -> LightingSettings {
    LightingSettings {
        preset: 1,
        brightness: 2,
        speed,
        color: None,
    }
}
fn complete(session: &mut Session, events: &Events) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(Event::Lighting {
            slot,
            device,
            result,
        }) = events.try_next()
        {
            session.finish(slot, device, result);
            return;
        }
        assert!(Instant::now() < deadline, "worker did not finish");
        std::thread::sleep(Duration::from_millis(1));
    }
}
#[test]
fn persistent_edits_never_enqueue_or_dispatch() {
    let calls = Arc::new(Mutex::new(vec![]));
    let keyboard = Keyboard {
        persistent: true,
        calls: calls.clone(),
        descriptor: DeviceDescriptor {
            name: "Synthetic persistent keyboard".into(),
            vendor: None,
            kind: DeviceKind::Keyboard,
            visual: None,
            details: vec![],
            driver: DriverInfo {
                id: "test",
                name: "Test",
                maturity: Maturity::Experimental,
                requirement: None,
            },
        },
    };
    let events = Events::new(Arc::new(|| {}));
    let mut session = Session::default();
    session.adopt(crate::app::events::Discovered {
        devices: vec![Box::new(keyboard)],
        dormant: vec![],
        notes: vec![],
        admitted: vec![],
    });
    for speed in 0..=4 {
        session.queue_lighting(0, settings(speed));
    }
    assert!(!session.live_lighting.pending());
    assert!(session.flush_lighting(&events).is_none());
    assert!(calls.lock().unwrap().is_empty());
    assert!(!session.entries()[0].busy());
}

#[test]
fn explicit_persistent_commit_waits_for_telemetry_without_losing_or_repeating_the_click() {
    let calls = Arc::new(Mutex::new(vec![]));
    let keyboard = Keyboard {
        persistent: true,
        calls: calls.clone(),
        descriptor: DeviceDescriptor {
            name: "Synthetic".into(),
            vendor: None,
            kind: DeviceKind::Keyboard,
            visual: None,
            details: vec![],
            driver: DriverInfo {
                id: "test",
                name: "Test",
                maturity: Maturity::Stable,
                requirement: None,
            },
        },
    };
    let events = Events::new(Arc::new(|| {}));
    let mut session = Session::default();
    session.adopt(crate::app::events::Discovered {
        devices: vec![Box::new(keyboard)],
        dormant: vec![],
        notes: vec![],
        admitted: vec![],
    });
    let device = session.entries[0].device.take();
    assert!(session.commit_lighting(0, settings(2), &events));
    assert!(session.entries()[0].lighting_pending());
    assert!(!session.commit_lighting(0, settings(4), &events));
    assert!(session.flush_lighting(&events).is_none());
    assert!(calls.lock().unwrap().is_empty());
    session.entries[0].device = device;
    session.flush_lighting(&events);
    complete(&mut session, &events);
    assert_eq!(*calls.lock().unwrap(), [2]);
    assert!(!session.entries()[0].lighting_pending());
    assert!(session.flush_lighting(&events).is_none());
    assert_eq!(*calls.lock().unwrap(), [2]);
}
#[test]
fn busy_worker_preserves_latest_edit_and_failure_never_retries() {
    let calls = Arc::new(Mutex::new(vec![]));
    let keyboard = Keyboard {
        persistent: false,
        calls: calls.clone(),
        descriptor: DeviceDescriptor {
            name: "Synthetic".into(),
            vendor: None,
            kind: DeviceKind::Keyboard,
            visual: None,
            details: vec![],
            driver: DriverInfo {
                id: "test",
                name: "Test",
                maturity: Maturity::Experimental,
                requirement: None,
            },
        },
    };
    let events = Events::new(Arc::new(|| {}));
    let mut session = Session::default();
    session.adopt(crate::app::events::Discovered {
        devices: vec![Box::new(keyboard)],
        dormant: vec![],
        notes: vec![],
        admitted: vec![],
    });
    // Merely opening or rendering an identified keyboard must not enqueue a write.
    assert!(session.flush_lighting(&events).is_none());
    assert!(calls.lock().unwrap().is_empty());
    session.queue_lighting(0, settings(0));
    session.flush_lighting(&events);
    session.queue_lighting(0, settings(1));
    session.queue_lighting(0, settings(2));
    assert!(session.flush_lighting(&events).is_none());
    complete(&mut session, &events);
    assert!(session.flush_lighting(&events).is_some());
    session.live_lighting.sent.get_mut(&0).unwrap().1 = Instant::now() - WRITE_INTERVAL;
    session.flush_lighting(&events);
    complete(&mut session, &events);
    assert_eq!(*calls.lock().unwrap(), vec![0, 2]);
    session.live_lighting.sent.get_mut(&0).unwrap().1 = Instant::now() - WRITE_INTERVAL;
    session.queue_lighting(0, settings(3));
    session.flush_lighting(&events);
    session.queue_lighting(0, settings(4));
    complete(&mut session, &events);
    assert!(session.flush_lighting(&events).is_none());
    assert_eq!(*calls.lock().unwrap(), vec![0, 2, 3]);
    assert!(matches!(
        session.entries()[0].status(),
        Some(super::super::Status::Failed(_))
    ));
}
