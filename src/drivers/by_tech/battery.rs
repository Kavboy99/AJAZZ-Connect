use super::{codec::Command, driver::read};
use crate::core::{Detail, DeviceDescriptor, DriverError, HidChannel, PowerSource};
use serde_json::Value;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Reading {
    percent: u8,
    status: u8,
}

impl Reading {
    pub(super) fn parse(bytes: &[u8]) -> Result<Self, DriverError> {
        if bytes.len() != 2 || bytes[0] > 100 {
            return Err(DriverError::protocol("invalid battery payload"));
        }
        Ok(Self {
            percent: bytes[0],
            status: bytes[1],
        })
    }

    pub(super) fn details(self) -> [Detail; 2] {
        let source = match self.status {
            0x01 => PowerSource::Battery,
            0x10 => PowerSource::External,
            _ => PowerSource::Unknown,
        };
        [Detail::Battery(self.percent), Detail::Power(source)]
    }

    pub(super) fn json(self) -> Value {
        serde_json::json!({"percent": self.percent, "status_raw": self.status,
            "power_source": match self.status { 1 => "battery", 16 => "external", _ => "unknown" },
            "charging": null, "provenance": "discovery_snapshot"})
    }
}

pub(super) fn wired(channel: &dyn HidChannel) -> Result<Reading, DriverError> {
    std::thread::sleep(Duration::from_millis(30));
    Reading::parse(&read(channel, Command::ReadPower)?)
}

/// Battery failure must not remove otherwise usable lighting controls.
pub(super) fn append(channel: &dyn HidChannel, descriptor: &mut DeviceDescriptor) {
    if let Ok(reading) = wired(channel) {
        descriptor.details.extend(reading.details());
    }
}

pub(super) fn identify(channel: &dyn HidChannel, details: &mut Value) {
    match wired(channel) {
        Ok(reading) => details["battery"] = reading.json(),
        Err(error) => details["battery_error"] = error.to_string().into(),
    }
}

impl crate::core::Battery for super::driver::ByTechDevice {
    fn refresh_battery(&mut self) -> Result<(), DriverError> {
        let exchange = self.exchange()?;
        let reading = wired(exchange.channel.as_ref())?;
        self.descriptor
            .details
            .retain(|d| !matches!(d, Detail::Battery(_) | Detail::Power(_)));
        self.descriptor.details.extend(reading.details());
        Ok(())
    }
}
