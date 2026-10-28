use super::discovery;
use super::events::{Discovered, DriverNote, Event, Events};
use crate::core::{Device, DeviceDescriptor, DriverError, LightingTraits, Outcome, Rgb};
use crate::storage::backups;
use std::path::PathBuf;

/// The last operation on a device, as the interface shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Running,
    Done(Outcome),
    Failed(DriverError),
}

/// A found device. While an operation runs, the device itself is on a worker thread and only its
/// passport stays here.
pub struct Entry {
    descriptor: DeviceDescriptor,
    lighting: Option<LightingTraits>,
    device: Option<Box<dyn Device>>,
    status: Option<Status>,
    /// Recognized, but its driver is experimental and off: nothing was sent to it.
    dormant: bool,
}

impl Entry {
    fn new(mut device: Box<dyn Device>) -> Self {
        Self {
            descriptor: device.descriptor().clone(),
            lighting: device.lighting().map(|l| l.traits()),
            device: Some(device),
            status: None,
            dormant: false,
        }
    }

    fn dormant(descriptor: DeviceDescriptor) -> Self {
        Self {
            descriptor,
            lighting: None,
            device: None,
            status: None,
            dormant: true,
        }
    }

    /// The same device across two searches: same driver, name and facts.
    fn same_as(&self, other: &DeviceDescriptor) -> bool {
        let d = &self.descriptor;
        d.driver.id == other.driver.id && d.name == other.name && d.details == other.details
    }

    pub fn descriptor(&self) -> &DeviceDescriptor {
        &self.descriptor
    }

    pub fn lighting(&self) -> Option<LightingTraits> {
        self.lighting
    }

    pub fn status(&self) -> Option<&Status> {
        self.status.as_ref()
    }

    pub fn busy(&self) -> bool {
        !self.dormant && self.device.is_none()
    }

    pub fn is_dormant(&self) -> bool {
        self.dormant
    }
}

#[derive(Default)]
pub struct Session {
    entries: Vec<Entry>,
    notes: Vec<DriverNote>,
    searched: bool,
    discovering: bool,
    /// What the last search admitted; an automatic search with the same set changes nothing.
    admitted: Option<Vec<String>>,
}

impl Session {
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn notes(&self) -> &[DriverNote] {
        &self.notes
    }

    pub fn searched(&self) -> bool {
        self.searched
    }

    pub fn discovering(&self) -> bool {
        self.discovering
    }

    /// A new search would drop devices that are on worker threads, so it waits for them.
    pub fn busy(&self) -> bool {
        self.discovering || self.entries.iter().any(Entry::busy)
    }

    /// The refresh button: always a full search.
    pub fn discover(&mut self, events: &Events, experimental: bool) {
        self.search(events, experimental, None);
    }

    /// After a device came or went: a full search only if a relevant interface changed.
    pub fn refresh(&mut self, events: &Events, experimental: bool) {
        let previous = self.admitted.clone();
        self.search(events, experimental, previous);
    }

    fn search(&mut self, events: &Events, experimental: bool, previous: Option<Vec<String>>) {
        if self.busy() {
            return;
        }
        self.discovering = true;
        events.spawn(move || {
            discovery::search(experimental, previous.as_deref())
                .map_or(Event::Unchanged, Event::Discovered)
        });
    }

    pub fn unchanged(&mut self) {
        self.discovering = false;
    }

    /// Replaces the device list. Dropping the old devices hands their lighting back. Returns,
    /// for every old slot, the new slot of the same device, if it is still there.
    pub fn adopt(&mut self, discovered: Discovered) -> Vec<Option<usize>> {
        self.discovering = false;
        self.searched = true;
        self.admitted = Some(discovered.admitted);
        let entries: Vec<Entry> = discovered
            .devices
            .into_iter()
            .map(Entry::new)
            .chain(discovered.dormant.into_iter().map(Entry::dormant))
            .collect();
        let moved = self
            .entries
            .iter()
            .map(|old| entries.iter().position(|new| old.same_as(new.descriptor())))
            .collect();
        self.entries = entries;
        self.notes = discovered.notes;
        moved
    }

    fn run(
        &mut self,
        slot: usize,
        events: &Events,
        job: impl FnOnce(&mut dyn Device) -> Result<Outcome, DriverError> + Send + 'static,
    ) {
        let Some(entry) = self.entries.get_mut(slot) else {
            return;
        };
        let Some(mut device) = entry.device.take() else {
            return;
        };
        entry.status = Some(Status::Running);
        events.spawn(move || {
            let result = job(device.as_mut());
            Event::Lighting {
                slot,
                device,
                result,
            }
        });
    }

    pub fn apply_solid(&mut self, slot: usize, color: Rgb, events: &Events) {
        self.run(slot, events, move |d| match d.lighting() {
            Some(lighting) => lighting.apply_solid(color),
            None => Err(DriverError::rejected("no lighting capability")),
        });
    }

    pub fn restore(&mut self, slot: usize, events: &Events) {
        self.run(slot, events, |d| match d.lighting() {
            Some(lighting) => lighting.restore(),
            None => Err(DriverError::NoRollback),
        });
    }

    pub fn restore_file(&mut self, slot: usize, path: PathBuf, events: &Events) {
        self.run(slot, events, move |d| {
            let snapshot = backups::load(&path).map_err(|_| DriverError::ForeignBackup)?;
            match d.lighting() {
                Some(lighting) => lighting.restore_snapshot(&snapshot),
                None => Err(DriverError::NoRollback),
            }
        });
    }

    pub fn finish(
        &mut self,
        slot: usize,
        device: Box<dyn Device>,
        result: Result<Outcome, DriverError>,
    ) {
        if let Some(entry) = self.entries.get_mut(slot) {
            entry.device = Some(device);
            entry.status = Some(match result {
                Ok(outcome) => Status::Done(outcome),
                Err(error) => Status::Failed(error),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Detail, DeviceKind, DriverInfo, Maturity};

    fn descriptor(name: &str) -> DeviceDescriptor {
        DeviceDescriptor {
            kind: DeviceKind::Keyboard,
            vendor: None,
            name: name.into(),
            driver: DriverInfo {
                id: "test",
                name: "Test",
                maturity: Maturity::Experimental,
                requirement: None,
            },
            details: vec![Detail::Usb { vid: 1, pid: 2 }],
            visual: None,
        }
    }

    fn found(names: &[&str]) -> Discovered {
        Discovered {
            devices: Vec::new(),
            dormant: names.iter().map(|n| descriptor(n)).collect(),
            notes: Vec::new(),
            admitted: names.iter().map(|n| format!("test/{n}")).collect(),
        }
    }

    #[test]
    fn a_new_search_keeps_track_of_each_device_and_dormant_ones_are_never_busy() {
        let mut session = Session::default();
        session.adopt(found(&["a", "b"]));
        assert!(
            session
                .entries()
                .iter()
                .all(|e| e.is_dormant() && !e.busy())
        );
        assert!(!session.busy());
        let moved = session.adopt(found(&["c", "b"]));
        assert_eq!(moved, vec![None, Some(1)]);
    }
}
