use super::*;
use crate::app::{
    battery::{BatteryMode, BatteryView},
    events::BatteryResult,
};
use crate::core::Detail;
use std::time::{Duration, Instant};
impl Session {
    pub fn last_battery_reading(&self, slot: usize) -> Option<(&[Detail], Instant)> {
        let clock = &self.entries.get(slot)?.battery_clock;
        Some((&clock.last_reading, clock.updated?))
    }
    pub fn battery_updated_at(&self, slot: usize) -> Option<Instant> {
        self.entries.get(slot).and_then(|e| e.battery_clock.updated)
    }
    /// At most one telemetry job exists at a time, including devices on shared receivers.
    pub fn poll_batteries(
        &mut self,
        events: &Events,
        mode: BatteryMode,
        view: BatteryView,
        force: bool,
    ) -> Option<Duration> {
        let now = Instant::now();
        for (slot, e) in self
            .entries
            .iter_mut()
            .enumerate()
            .filter(|(_, e)| e.battery)
        {
            let interval = mode.interval(view, slot, &e.descriptor.details);
            e.battery_clock.configure(now, interval, force, slot);
        }
        if !self.entries.iter().any(|e| e.battery) {
            return None;
        }
        if self.discovering || self.entries.iter().any(|e| e.battery_clock.in_flight) {
            return Some(Duration::from_secs(3));
        }
        let candidate = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.battery && e.device.is_some())
            .min_by_key(|(_, e)| e.battery_clock.due)
            .map(|(slot, _)| slot);
        let slot = candidate?;
        let entry = &mut self.entries[slot];
        if entry.battery_clock.due > now {
            return Some(entry.battery_clock.due.duration_since(now));
        }
        let mut device = entry.device.take()?;
        entry.battery_clock.in_flight = true;
        events.spawn(move || {
            let result = device
                .battery()
                .ok_or_else(|| DriverError::rejected("no battery capability"))
                .and_then(|b| b.refresh_battery());
            Event::Battery(vec![BatteryResult {
                slot,
                device,
                result,
            }])
        });
        Some(Duration::from_secs(3))
    }
    pub fn finish_battery(&mut self, completed: BatteryResult) {
        if let Some(entry) = self.entries.get_mut(completed.slot) {
            entry
                .battery_clock
                .complete(Instant::now(), completed.result.is_ok());
            entry.descriptor = completed.device.descriptor().clone();
            if completed.result.is_ok() {
                entry.battery_clock.last_reading = entry
                    .descriptor
                    .details
                    .iter()
                    .filter(|d| matches!(d, Detail::Battery(_) | Detail::Power(_)))
                    .cloned()
                    .collect();
            }
            if completed.result.is_err() {
                entry
                    .descriptor
                    .details
                    .retain(|d| !matches!(d, Detail::Battery(_) | Detail::Power(_)));
            }
            entry.device = Some(completed.device);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Battery, DeviceKind, DriverInfo, Maturity};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    struct Mock {
        descriptor: DeviceDescriptor,
        fail: bool,
        count: Arc<AtomicUsize>,
    }
    impl Device for Mock {
        fn descriptor(&self) -> &DeviceDescriptor {
            &self.descriptor
        }
        fn battery(&mut self) -> Option<&mut dyn Battery> {
            Some(self)
        }
    }
    impl Battery for Mock {
        fn refresh_battery(&mut self) -> Result<(), DriverError> {
            self.count.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                return Err(DriverError::Changed);
            }
            self.descriptor.details = vec![Detail::Battery(49)];
            Ok(())
        }
    }
    fn mock(fail: bool, count: Arc<AtomicUsize>) -> Entry {
        Entry::new(Box::new(Mock {
            descriptor: DeviceDescriptor {
                name: "test".into(),
                vendor: None,
                visual: None,
                kind: DeviceKind::Keyboard,
                driver: DriverInfo {
                    id: "test",
                    name: "test",
                    maturity: Maturity::Stable,
                    requirement: None,
                },
                details: vec![Detail::Battery(50)],
            },
            fail,
            count,
        }))
    }
    fn complete(session: &mut Session, events: &Events, rx: &std::sync::mpsc::Receiver<()>) {
        rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let Event::Battery(results) = events.try_next().unwrap() else {
            panic!("wrong event")
        };
        assert_eq!(results.len(), 1);
        for result in results {
            session.finish_battery(result);
        }
    }
    #[test]
    fn one_job_independent_failure_and_retained_freshness() {
        let count = Arc::new(AtomicUsize::new(0));
        let mut session = Session {
            entries: vec![mock(true, count.clone()), mock(false, count.clone())],
            ..Default::default()
        };
        let (tx, rx) = std::sync::mpsc::channel();
        let events = Events::new(Arc::new(move || {
            let _ = tx.send(());
        }));
        session.poll_batteries(
            &events,
            BatteryMode::Automatic,
            BatteryView::Device(0),
            true,
        );
        session.poll_batteries(
            &events,
            BatteryMode::Automatic,
            BatteryView::Device(0),
            false,
        );
        complete(&mut session, &events, &rx);
        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert!(session.entries[0].descriptor.details.is_empty());
        assert_eq!(
            session.last_battery_reading(0).unwrap().0,
            &[Detail::Battery(50)]
        );
        session.entries[1].battery_clock.due = Instant::now();
        session.poll_batteries(
            &events,
            BatteryMode::Automatic,
            BatteryView::Device(0),
            false,
        );
        complete(&mut session, &events, &rx);
        assert_eq!(count.load(Ordering::SeqCst), 2);
        assert!(
            session.entries[1]
                .descriptor
                .details
                .contains(&Detail::Battery(49))
        );
        assert!(session.entries[1].status.is_none());
        assert!(!session.busy());
    }
}
