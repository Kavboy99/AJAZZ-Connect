use super::{DriverError, Outcome, Snapshot};

/// The driver declares exact coverage; a partial snapshot must never be advertised as a memory dump.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackupCoverage {
    SystemAndPalette,
    Lighting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreScope {
    SystemConfiguration,
    Lighting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackupInfo {
    pub coverage: BackupCoverage,
    pub restore: Option<RestoreScope>,
}

/// Independent of lighting writes: recovery can remain available when an editor is quarantined.
pub trait ConfigurationBackup {
    fn backup_info(&self) -> BackupInfo;
    fn capture(&mut self) -> Result<Snapshot, DriverError>;
    /// Pure validation against the identified device, with no IO or changes.
    fn check_backup(&self, snapshot: &Snapshot) -> Result<(), DriverError>;
    /// Explicit user action only; saves current state first and verifies read-back. Never retries.
    fn recover(&mut self, snapshot: &Snapshot) -> Result<Outcome, DriverError>;
}
