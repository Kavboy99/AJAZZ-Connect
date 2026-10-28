use super::{DriverError, Rgb, Snapshot};

/// How a lighting change can be undone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rollback {
    /// The protocol cannot read the original state back.
    Unavailable,
    /// The state saved before the first change in this session can be restored.
    Session,
    /// As `Session`, plus a backup file written before the first change.
    SessionAndFile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightingTraits {
    /// The device can take a solid color right now.
    pub writable: bool,
    /// The change is stored in device memory and survives power loss.
    pub persistent: bool,
    pub rollback: Rollback,
}

/// What a successful operation proved. The interface words each case differently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The state read back matches what was written.
    Confirmed,
    /// The mode read back matches, but this firmware reports a palette instead of the color.
    ModeConfirmed,
    /// Every packet was acknowledged; the protocol has no read-back.
    Acknowledged,
    /// The packets were sent; the standard defines no acknowledgement.
    Sent { lamps: usize },
    /// A local service confirmed the new state in its own model.
    ServiceConfirmed,
    /// The state from before the first change is back.
    Restored,
    /// Control was handed back to the device's own lighting program.
    Released,
}

/// Solid-color lighting. A device that has no lighting simply does not return this capability.
pub trait Lighting {
    fn traits(&self) -> LightingTraits;
    fn apply_solid(&mut self, color: Rgb) -> Result<Outcome, DriverError>;
    /// Undo the changes made in this session.
    fn restore(&mut self) -> Result<Outcome, DriverError>;
    /// Restore from a backup file; only for `Rollback::SessionAndFile`.
    fn restore_snapshot(&mut self, snapshot: &Snapshot) -> Result<Outcome, DriverError>;
}
