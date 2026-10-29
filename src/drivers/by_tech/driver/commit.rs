//! Explicit, backed-up lighting transactions. Each changed block is verified before proceeding.
use super::super::{
    backup::Backup,
    codec, lighting,
    state::{ColorTable, Config},
};
use super::{ByTechDevice, COMMAND_GAP, Command, read};
use crate::core::{DriverError, HidChannel, LightingSettings, Outcome};
use std::time::Duration;

const STORE_DELAY: Duration = Duration::from_millis(500);

fn write_verified(
    channel: &dyn HidChannel,
    write: Command,
    verify: Command,
    data: &[u8],
) -> Result<(), DriverError> {
    let packet = codec::write(write, data)?;
    crate::diagnostics::logging::command(super::super::ID, packet[1], true);
    channel
        .send_feature(&packet)
        .map_err(|e| DriverError::Unconfirmed(e.to_string()))?;
    std::thread::sleep(STORE_DELAY);
    if read(channel, verify)? != data {
        return Err(DriverError::ReadBackMismatch);
    }
    Ok(())
}

impl ByTechDevice {
    pub(super) fn can_commit(&self) -> bool {
        self.protocol_identity == "03000000015e-0100"
            && self.cached.is_some()
            && self.cached_colors.is_some()
    }

    pub(super) fn commit(&mut self, settings: LightingSettings) -> Result<Outcome, DriverError> {
        if !self.can_commit() {
            return Err(DriverError::rejected(
                "lighting configuration or hardware revision is not supported",
            ));
        }
        // Reject unsupported inputs before opening the channel.
        let cached = (
            self.cached.as_ref().ok_or(DriverError::Changed)?.clone(),
            self.cached_colors
                .as_ref()
                .ok_or(DriverError::Changed)?
                .clone(),
        );
        lighting::target(&cached, settings)?;
        let exchange = self.exchange()?;
        let channel = exchange.channel.as_ref();
        let config = Config::parse(&read(channel, Command::ReadConfig)?)?;
        std::thread::sleep(COMMAND_GAP);
        let colors = ColorTable::parse(&read(channel, Command::ReadColors)?)?;
        let current = (config, colors);
        let target = lighting::target(&current, settings)?;
        if target != current {
            let backup = Backup::new(
                &self.identity,
                &self.protocol_identity,
                &current.0,
                &current.1,
            );
            self.bus.store_backup(
                "by-tech-before-lighting",
                &self.identity,
                &backup.to_json()?,
            )?;
            // Persist and verify the palette separately; a failed palette must never select the mode.
            if target.1 != current.1 {
                std::thread::sleep(COMMAND_GAP);
                write_verified(
                    channel,
                    Command::WriteColors,
                    Command::ReadColors,
                    target.1.bytes(),
                )?;
            }
            std::thread::sleep(COMMAND_GAP);
            if read(channel, Command::ReadConfig)? != current.0.bytes() {
                return Err(DriverError::Changed);
            }
            if target.0 != current.0 {
                std::thread::sleep(COMMAND_GAP);
                write_verified(
                    channel,
                    Command::WriteConfig,
                    Command::ReadConfig,
                    target.0.bytes(),
                )?;
            }
            // A later configuration write must not have disturbed the already verified palette.
            std::thread::sleep(COMMAND_GAP);
            if read(channel, Command::ReadColors)? != target.1.bytes() {
                return Err(DriverError::ReadBackMismatch);
            }
        }
        self.cached = Some(target.0);
        self.cached_colors = Some(target.1);
        Ok(Outcome::Confirmed)
    }
}
