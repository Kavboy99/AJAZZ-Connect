use super::{Event, Events, Session, Status};
use crate::core::{Device, DriverError, Outcome, Rgb};
use crate::storage::backups;
use std::path::PathBuf;

impl Session {
    pub(super) fn run(
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
        let driver = entry.descriptor.driver.id;
        events.spawn(move || {
            crate::diagnostics::logging::event("operation", driver, "start");
            let result = job(device.as_mut());
            crate::diagnostics::logging::event(
                "operation",
                driver,
                result
                    .as_ref()
                    .err()
                    .map_or("ok", crate::diagnostics::logging::error_code),
            );
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

    pub fn apply_lighting(
        &mut self,
        slot: usize,
        settings: crate::core::LightingSettings,
        events: &Events,
    ) {
        self.run(slot, events, move |d| match d.lighting() {
            Some(lighting) => lighting.apply_settings(settings),
            None => Err(DriverError::rejected("no lighting capability")),
        });
    }

    pub fn restore(&mut self, slot: usize, events: &Events) {
        if let Some(entry) = self.entries.get_mut(slot) {
            entry.pending_lighting = None;
        }
        self.live_lighting.cancel(slot);
        self.run(slot, events, |d| match d.lighting() {
            Some(lighting) => lighting.restore(),
            None => Err(DriverError::NoRollback),
        });
    }

    pub fn restore_file(&mut self, slot: usize, path: PathBuf, events: &Events) {
        if let Some(entry) = self.entries.get_mut(slot) {
            entry.pending_lighting = None;
        }
        self.live_lighting.cancel(slot);
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
        mut device: Box<dyn Device>,
        result: Result<Outcome, DriverError>,
    ) {
        if result.is_err() {
            self.live_lighting.cancel(slot);
        }
        if let Some(entry) = self.entries.get_mut(slot) {
            if result.is_err() {
                entry.pending_lighting = None;
            }
            entry.editor = device.lighting().and_then(|l| l.editor());
            entry.device = Some(device);
            entry.status = Some(match result {
                Ok(outcome) => Status::Done(outcome),
                Err(error) => Status::Failed(error),
            });
        }
    }
}
