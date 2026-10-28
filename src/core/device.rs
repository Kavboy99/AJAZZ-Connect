use super::DriverInfo;
use serde::{Deserialize, Serialize};

/// What a device is to the user. Decides the icon and the wording, never the protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    Keyboard,
    Mouse,
    Headset,
    Gamepad,
    Lighting,
    Component,
    Other,
}

/// One fact about a device, shown on its page. Structured so the interface can translate it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Detail {
    Usb { vid: u16, pid: u16 },
    Firmware(String),
    Release(u16),
    Leds(usize),
}

/// The device passport: everything the interface needs to list and describe a device.
#[derive(Clone, Debug)]
pub struct DeviceDescriptor {
    pub kind: DeviceKind,
    pub vendor: Option<String>,
    pub name: String,
    pub driver: DriverInfo,
    pub details: Vec<Detail>,
    /// The id of a 3D model of the device, from its profile. The interface draws the kind icon
    /// when there is none.
    pub visual: Option<String>,
}
