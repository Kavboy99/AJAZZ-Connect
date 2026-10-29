use super::{Events, Session};
use crate::{
    core::{DriverError, Outcome, Snapshot},
    storage::archive,
};
use std::path::PathBuf;

impl Session {
    pub fn discover_backups(&mut self, events: &Events) {
        if self.busy() {
            return;
        }
        self.discovering = true;
        events.spawn(|| crate::app::Event::Discovered(crate::app::backup_discovery::search()));
    }
    pub fn export_backup(&mut self, slot: usize, path: PathBuf, events: &Events) {
        if let Some(entry) = self.entries.get_mut(slot) {
            entry.pending_lighting = None;
        }
        crate::diagnostics::logging::event("action", "backup.export", "requested");
        self.live_lighting.cancel(slot);
        self.run(slot, events, move |device| {
            let backup = device
                .configuration_backup()
                .ok_or(DriverError::NoRollback)?;
            let snapshot = backup.capture()?;
            archive::save(&path, &snapshot)?;
            Ok(Outcome::BackupSaved)
        });
    }
    pub fn check_backup(&mut self, slot: usize, snapshot: &Snapshot) -> Result<(), DriverError> {
        self.entries
            .get_mut(slot)
            .and_then(|e| e.device.as_mut())
            .and_then(|d| d.configuration_backup())
            .ok_or(DriverError::Busy)?
            .check_backup(snapshot)
    }
    /// The UI must obtain a direct restore-button press after showing the validated archive scope.
    pub fn recover_backup(&mut self, slot: usize, snapshot: Snapshot, events: &Events) {
        if let Some(entry) = self.entries.get_mut(slot) {
            entry.pending_lighting = None;
        }
        crate::diagnostics::logging::event("action", "backup.recover", "confirmed");
        self.live_lighting.cancel(slot);
        self.run(slot, events, move |device| {
            device
                .configuration_backup()
                .ok_or(DriverError::NoRollback)?
                .recover(&snapshot)
        });
    }
}
