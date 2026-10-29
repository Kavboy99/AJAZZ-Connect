//! Owner-authorized explicit lighting commits; drafts never reach this adapter.
use super::ByTechDevice;
use crate::core::{DriverError, Lighting, LightingTraits, Outcome, Rgb, Rollback, Snapshot};

impl Lighting for ByTechDevice {
    fn editor(&self) -> Option<crate::core::LightingEditor> {
        Some(super::super::lighting::editor(
            self.cached.as_ref()?,
            self.cached_colors.as_ref()?,
        ))
    }
    fn traits(&self) -> LightingTraits {
        LightingTraits {
            writable: self.can_commit(),
            persistent: true,
            rollback: Rollback::Unavailable,
        }
    }
    fn apply_settings(
        &mut self,
        settings: crate::core::LightingSettings,
    ) -> Result<Outcome, DriverError> {
        self.commit(settings)
    }
    fn apply_solid(&mut self, color: Rgb) -> Result<Outcome, DriverError> {
        let config = self.cached.as_ref().ok_or(DriverError::Changed)?;
        let (speed, brightness, _) = config.settings(1);
        self.commit(crate::core::LightingSettings {
            preset: 1,
            speed,
            brightness,
            color: Some(color),
        })
    }
    fn restore(&mut self) -> Result<Outcome, DriverError> {
        Err(DriverError::NoRollback)
    }
    fn restore_snapshot(&mut self, _: &Snapshot) -> Result<Outcome, DriverError> {
        Err(DriverError::NoRollback)
    }
}
