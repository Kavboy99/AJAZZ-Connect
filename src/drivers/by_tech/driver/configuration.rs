use super::super::{
    backup::Backup,
    state::{ColorTable, Config},
};
use super::{ByTechDevice, COMMAND_GAP, Command, codec, read};
use crate::core::{
    BackupCoverage, BackupInfo, ConfigurationBackup, DriverError, Outcome, RestoreScope, Snapshot,
};
use std::time::Duration;

const VERIFIED_ID: &str = "03000000015e-0100";
const DESCRIPTOR: &str =
    "0600ff0901a10185060902150026ff0075089507810019012902150026ff007508960702b102c0";

impl ByTechDevice {
    fn raw_snapshot(&self, channel: &dyn crate::core::HidChannel) -> Result<Snapshot, DriverError> {
        let config = read(channel, Command::ReadConfig)?;
        std::thread::sleep(COMMAND_GAP);
        let colors = read(channel, Command::ReadColors)?;
        Snapshot::from_bytes(
            Backup::from_raw(&self.identity, &self.protocol_identity, config, colors).to_json()?,
        )
    }
}

impl ConfigurationBackup for ByTechDevice {
    fn backup_info(&self) -> BackupInfo {
        BackupInfo {
            coverage: BackupCoverage::SystemAndPalette,
            restore: (self.protocol_identity == VERIFIED_ID)
                .then_some(RestoreScope::SystemConfiguration),
        }
    }
    fn capture(&mut self) -> Result<Snapshot, DriverError> {
        let exchange = self.exchange()?;
        self.raw_snapshot(exchange.channel.as_ref())
    }
    fn check_backup(&self, snapshot: &Snapshot) -> Result<(), DriverError> {
        if self.protocol_identity != VERIFIED_ID {
            return Err(DriverError::rejected(
                "configuration recovery is not verified for this revision",
            ));
        }
        let backup: Backup = snapshot.parse()?;
        backup.recovery_config(&self.identity, &self.protocol_identity)?;
        Ok(())
    }
    fn recover(&mut self, snapshot: &Snapshot) -> Result<Outcome, DriverError> {
        self.check_backup(snapshot)?;
        let backup: Backup = snapshot.parse()?;
        let target = backup.recovery_config(&self.identity, &self.protocol_identity)?;
        let exchange = self.exchange()?;
        let channel = exchange.channel.as_ref();
        let mut descriptor = [0; 4096];
        let count = channel.report_descriptor(&mut descriptor)?;
        if descriptor.get(..count).map(codec::hex).as_deref() != Some(DESCRIPTOR) {
            return Err(DriverError::Changed);
        }
        // Preserve even an invalid source block, because recovery is needed precisely in that state.
        let before = self.raw_snapshot(channel)?;
        self.bus
            .store_backup("before-recovery", &self.identity, before.bytes())?;
        std::thread::sleep(COMMAND_GAP);
        let current = read(channel, Command::ReadConfig)?;
        if current == target.bytes() {
            self.cached = Some(target);
            return Ok(Outcome::ConfigurationRestored);
        }
        std::thread::sleep(COMMAND_GAP);
        // Recovery intentionally restores only the owner-verified system block, never the palette.
        crate::diagnostics::logging::command(super::super::ID, 4, true);
        channel
            .send_feature(&codec::write(Command::WriteConfig, target.bytes())?)
            .map_err(|e| DriverError::Unconfirmed(e.to_string()))?;
        std::thread::sleep(Duration::from_millis(500));
        let observed = read(channel, Command::ReadConfig)?;
        if observed != target.bytes() {
            self.cached = Config::parse(&observed).ok();
            return Err(DriverError::ReadBackMismatch);
        }
        self.cached = Some(target);
        // Cache refresh uses a read only; the archived palette is never restored.
        std::thread::sleep(COMMAND_GAP);
        self.cached_colors = ColorTable::parse(&read(channel, Command::ReadColors)?).ok();
        Ok(Outcome::ConfigurationRestored)
    }
}
