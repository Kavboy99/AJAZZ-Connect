//! Synthetic states for `--review`. These devices exist only in memory: they never open HID or a
//! socket, and every operation on them fails.

use crate::app::events::{Discovered, DriverNote};
use crate::core::{
    Detail, Device, DeviceDescriptor, DeviceKind, DriverError, DriverInfo, Endpoint, Lighting,
    LightingTraits, Maturity, Outcome, ReportSizes, Requirement, Rgb, Rollback, Snapshot,
};
use crate::diagnostics::{Interface, Report};

struct ReviewDevice {
    descriptor: DeviceDescriptor,
    traits: LightingTraits,
    programs: Option<crate::core::LightingEditor>,
    lighting_available: bool,
}

impl Device for ReviewDevice {
    fn descriptor(&self) -> &DeviceDescriptor {
        &self.descriptor
    }

    fn lighting(&mut self) -> Option<&mut dyn Lighting> {
        self.lighting_available.then_some(self)
    }
}

fn offline() -> DriverError {
    DriverError::Unavailable("review mockup".into())
}

impl Lighting for ReviewDevice {
    fn editor(&self) -> Option<crate::core::LightingEditor> {
        self.programs.clone()
    }
    fn traits(&self) -> LightingTraits {
        self.traits
    }

    fn apply_solid(&mut self, _: Rgb) -> Result<Outcome, DriverError> {
        Err(offline())
    }

    fn restore(&mut self) -> Result<Outcome, DriverError> {
        Err(offline())
    }

    fn restore_snapshot(&mut self, _: &Snapshot) -> Result<Outcome, DriverError> {
        Err(offline())
    }
}

fn driver(id: &'static str, name: &'static str, maturity: Maturity) -> DriverInfo {
    DriverInfo {
        id,
        name,
        maturity,
        requirement: None,
    }
}

fn device(
    kind: DeviceKind,
    vendor: Option<&str>,
    name: &str,
    info: DriverInfo,
    details: Vec<Detail>,
    traits: LightingTraits,
    visual: Option<&str>,
) -> Box<dyn Device> {
    Box::new(ReviewDevice {
        programs: None,
        lighting_available: true,
        descriptor: DeviceDescriptor {
            kind,
            vendor: vendor.map(str::to_owned),
            name: name.into(),
            driver: info,
            details,
            visual: visual.map(str::to_owned),
        },
        traits,
    })
}

pub(super) fn native_devices() -> Discovered {
    native_devices_for(0)
}

pub(super) fn native_devices_for(current: u8) -> Discovered {
    use crate::core::{LightingEditor, LightingEffect as E, LightingPreset, LightingSettings};
    let mut found = devices();
    let descriptor = found.devices[0].descriptor().clone();
    let presets = [
        E::Solid,
        E::Breathing,
        E::Rainbow,
        E::Flash,
        E::Rain,
        E::RainbowWheel,
        E::Ripples,
        E::Stars,
        E::Fade,
        E::Snake,
        E::Neon,
        E::Reactive,
        E::Sine,
        E::Scan,
        E::Windmill,
        E::Waterfall,
        E::Blossom,
        E::Storm,
        E::Off,
    ]
    .into_iter()
    .enumerate()
    .map(|(i, effect)| LightingPreset {
        effect,
        saved: LightingSettings {
            preset: i as u8,
            brightness: 3,
            speed: 2,
            color: None,
        },
        brightness_max: (effect != E::Off).then_some(4),
        speed_max: (!matches!(effect, E::Off | E::Solid)).then_some(4),
        custom_color: !matches!(effect, E::Off | E::Rainbow),
    })
    .collect();
    found.devices[0] = Box::new(ReviewDevice {
        lighting_available: true,
        descriptor,
        traits: LightingTraits {
            writable: true,
            persistent: true,
            rollback: Rollback::SessionAndFile,
        },
        programs: Some(LightingEditor { presets, current }),
    });
    found
}

pub(super) fn connection_devices(driver: DriverInfo) -> Discovered {
    let mut found = devices();
    let mut descriptor = found.devices[0].descriptor().clone();
    descriptor.driver = driver;
    let traits = LightingTraits {
        writable: false,
        persistent: true,
        rollback: Rollback::Unavailable,
    };
    found.devices[0] = Box::new(ReviewDevice {
        descriptor,
        traits,
        programs: None,
        lighting_available: false,
    });
    found
}

pub fn devices() -> Discovered {
    let persistent = LightingTraits {
        writable: true,
        persistent: true,
        rollback: Rollback::SessionAndFile,
    };
    let volatile = LightingTraits {
        writable: true,
        persistent: false,
        rollback: Rollback::Session,
    };
    let feature = LightingTraits {
        writable: true,
        persistent: true,
        rollback: Rollback::Unavailable,
    };
    let openrgb = DriverInfo {
        requirement: Some(Requirement::LocalService {
            name: "OpenRGB",
            address: "127.0.0.1:6742",
        }),
        ..driver("openrgb", "OpenRGB SDK", Maturity::Stable)
    };
    Discovered {
        devices: vec![
            device(
                DeviceKind::Keyboard,
                Some("Vendor"),
                "Keyboard (mockup)",
                driver("sonix_aa55", "SONiX AA55", Maturity::Stable),
                vec![
                    Detail::Usb {
                        vid: 0x0c45,
                        pid: 0x80a0,
                    },
                    Detail::Firmware("1.36".into()),
                    Detail::Battery(79),
                    Detail::Power(crate::core::PowerSource::External),
                ],
                persistent,
                Some("ak820-max"),
            ),
            device(
                DeviceKind::Mouse,
                None,
                "Mouse (mockup)",
                driver("hid_lamparray", "HID LampArray", Maturity::Stable),
                vec![
                    Detail::Usb {
                        vid: 0x1234,
                        pid: 0x5678,
                    },
                    Detail::Leds(12),
                ],
                volatile,
                None,
            ),
            device(
                DeviceKind::Keyboard,
                Some("Vendor"),
                "Keyboard, older board (mockup)",
                driver("sonix_feature", "SONiX Feature", Maturity::Experimental),
                vec![
                    Detail::Usb {
                        vid: 0x0c45,
                        pid: 0x8009,
                    },
                    Detail::Release(0x0105),
                ],
                feature,
                None,
            ),
        ],
        dormant: vec![DeviceDescriptor {
            kind: DeviceKind::Keyboard,
            vendor: Some("Vendor".into()),
            name: "Keyboard, driver off (mockup)".into(),
            driver: driver("by_tech", "BY Tech (BYCOMBO4)", Maturity::Experimental),
            details: vec![Detail::Usb {
                vid: 0x258a,
                pid: 0x010c,
            }],
            visual: Some("ak820-max".into()),
        }],
        admitted: Vec::new(),
        notes: vec![DriverNote {
            driver: Some(openrgb),
            subject: None,
            error: DriverError::Unavailable("127.0.0.1:6742 refused".into()),
        }],
    }
}

/// A report shaped like the passive probe of a BY Tech keyboard (`tests/fixtures`).
pub fn report() -> Report {
    let sizes = |input, output, feature| {
        Some(ReportSizes {
            input,
            output,
            feature,
        })
    };
    let mut report = Report::empty();
    for (vid, pid, page, usage, product, reports) in [
        (
            0x258a,
            0x010c,
            0x0001,
            0x0006,
            "Gaming Keyboard",
            sizes(9, 2, 0),
        ),
        (
            0x258a,
            0x010c,
            0xff00,
            0x0001,
            "Gaming Keyboard",
            sizes(8, 0, 520),
        ),
        (
            0x3554,
            0xfa09,
            0xff02,
            0x0002,
            "2.4G Wireless Receiver",
            sizes(20, 20, 0),
        ),
    ] {
        let endpoint = Endpoint::hid(vid, pid, page, usage)
            .with_product(product)
            .with_reports(reports)
            .with_path(format!("review-{vid:04x}-{pid:04x}-{page:04x}").into_bytes());
        report.interfaces.push(Interface::of(&endpoint));
    }
    report
}
