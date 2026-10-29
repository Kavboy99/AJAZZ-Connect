use super::{Aa55Device, Backup, GET_LED_EFFECT, codec, transaction};
use crate::core::{
    BackupCoverage, BackupInfo, ConfigurationBackup, DriverError, Outcome, RestoreScope, Snapshot,
};

impl ConfigurationBackup for Aa55Device {
    fn backup_info(&self) -> BackupInfo {
        BackupInfo {
            coverage: BackupCoverage::Lighting,
            restore: Some(RestoreScope::Lighting),
        }
    }
    fn capture(&mut self) -> Result<Snapshot, DriverError> {
        let exchange = self.exchange()?;
        let payload = transaction(exchange.channel.as_ref(), GET_LED_EFFECT, None)?;
        codec::validate_lighting(&payload)?;
        Snapshot::from_bytes(
            Backup::new(&self.identity, &self.protocol_identity, payload).to_json()?,
        )
    }
    fn check_backup(&self, snapshot: &Snapshot) -> Result<(), DriverError> {
        let backup: Backup = snapshot.parse()?;
        backup.check(&self.identity, &self.protocol_identity)?;
        codec::validate_lighting(&backup.lighting)
    }
    fn recover(&mut self, snapshot: &Snapshot) -> Result<Outcome, DriverError> {
        self.check_backup(snapshot)?;
        let target: Backup = snapshot.parse()?;
        let exchange = self.exchange()?;
        let channel = exchange.channel.as_ref();
        let current = transaction(channel, GET_LED_EFFECT, None)?;
        codec::validate_lighting(&current)?;
        let before = Backup::new(&self.identity, &self.protocol_identity, current);
        self.bus
            .store_backup("before-recovery-aa55", &self.identity, &before.to_json()?)?;
        self.write_lighting(channel, &target.lighting)
            .map(Self::restored)
    }
}
