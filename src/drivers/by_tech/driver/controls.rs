//! Native and legacy lighting operations share the same backup and verification path.
use super::super::backup::Backup;
use super::ByTechDevice;
use crate::core::{DriverError, Lighting, LightingTraits, Outcome, Rgb, Rollback, Snapshot};

impl Lighting for ByTechDevice {
    fn editor(&self) -> Option<crate::core::LightingEditor> {
        Some(super::super::lighting::editor(&self.cached))
    }

    fn apply_settings(
        &mut self,
        settings: crate::core::LightingSettings,
    ) -> Result<Outcome, DriverError> {
        self.apply_target(Some(settings), Rgb::default())
    }
    fn traits(&self) -> LightingTraits {
        LightingTraits {
            writable: true,
            persistent: true,
            rollback: Rollback::SessionAndFile,
        }
    }

    fn apply_solid(&mut self, color: Rgb) -> Result<Outcome, DriverError> {
        self.apply_target(None, color)
    }

    fn restore(&mut self) -> Result<Outcome, DriverError> {
        let original = self.original.clone().ok_or(DriverError::NoRollback)?;
        self.restore_to(&original)
    }

    fn restore_snapshot(&mut self, snapshot: &Snapshot) -> Result<Outcome, DriverError> {
        let backup: Backup = snapshot.parse()?;
        let target = backup.blocks(&self.identity, &self.protocol_identity)?;
        self.restore_to(&target)
    }
}
