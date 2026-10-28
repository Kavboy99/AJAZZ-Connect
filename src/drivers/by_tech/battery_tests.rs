use super::{
    battery::Reading,
    codec::{self, Command},
    radio, radio_codec, *,
};
use crate::core::{Bus, Detail, Driver, PowerSource, ReportSizes};
use crate::drivers::fake_bus::{FakeBus, Packet};
use std::sync::Arc;

fn endpoint() -> Endpoint {
    Endpoint::hid(0x3554, 0xfa09, 0xff02, 2)
        .with_release(0x2008)
        .with_reports(Some(ReportSizes {
            input: 20,
            output: 20,
            feature: 0,
        }))
}
fn captured() -> Vec<u8> {
    vec![
        0x13, 0x4a, 1, 0, 2, 0x4d, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xae,
    ]
}
fn bus(reply: Vec<u8>) -> Arc<FakeBus> {
    let fake = FakeBus::new(
        vec![endpoint()],
        Box::new(move |packet| {
            assert_eq!(packet, &Packet::Output(radio_codec::request().to_vec()));
            vec![reply.clone()]
        }),
    );
    fake.set_descriptor(radio_codec::DESCRIPTOR.to_vec());
    fake
}
#[test]
fn captured_battery_samples_and_unknown_status() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/by-tech-battery.json")).unwrap();
    for sample in data.as_array().unwrap() {
        let bytes: Vec<u8> = sample["reply"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u8)
            .collect();
        let reading = if sample["transport"] == "usb" {
            Reading::parse(codec::reply(&bytes, Command::ReadPower).unwrap()).unwrap()
        } else {
            radio_codec::reply(&bytes).unwrap()
        };
        assert_eq!(reading.json()["percent"], sample["percent"]);
        assert!(reading.json()["charging"].is_null());
    }
    assert_eq!(
        Reading::parse(&[42, 0xff]).unwrap().details()[1],
        Detail::Power(PowerSource::Unknown)
    );
    assert!(Reading::parse(&[101, 1]).is_err());
    assert!(Reading::parse(&[]).is_err());
    assert_eq!(
        &codec::request(Command::ReadPower).unwrap()[..8],
        &[6, 0x87, 2, 0, 1, 0, 2, 0]
    );
    let mut stale = vec![6, 0x84, 2, 0, 1, 0, 2, 0, 77, 1];
    assert!(codec::reply(&stale, Command::ReadPower).is_err());
    stale[1] = 0x87;
    stale[2] = 0;
    assert!(codec::reply(&stale, Command::ReadPower).is_err());
}
#[test]
fn radio_is_read_only_and_rejects_malformed_frames() {
    let fake = bus(captured());
    let shared: Arc<dyn Bus> = fake.clone();
    let mut found = ByTechDriver.discover(&shared);
    assert_eq!(found.devices.len(), 1, "{:?}", found.notes);
    assert!(found.devices[0].lighting().is_none());
    assert!(
        found.devices[0]
            .descriptor()
            .details
            .contains(&Detail::Battery(77))
    );
    assert_eq!(
        fake.log(),
        vec![Packet::Output(radio_codec::request().to_vec())]
    );
    for index in [0, 1, 2, 3, 4, 5, 19] {
        let mut bad = captured();
        bad[index] ^= 1;
        assert!(radio_codec::reply(&bad).is_err());
    }
    assert!(radio_codec::reply(&captured()[..19]).is_err());
}
#[test]
fn foreign_descriptor_is_rejected_and_timeout_keeps_confirmed_receiver() {
    let fake = bus(captured());
    fake.set_descriptor(vec![0; 36]);
    let shared: Arc<dyn Bus> = fake.clone();
    assert!(ByTechDriver.discover(&shared).devices.is_empty());
    assert!(fake.log().is_empty());
    let fake = FakeBus::new(vec![endpoint()], Box::new(|_| Vec::new()));
    fake.set_descriptor(radio_codec::DESCRIPTOR.to_vec());
    let shared: Arc<dyn Bus> = fake.clone();
    let mut found = ByTechDriver.discover(&shared);
    assert_eq!(found.devices.len(), 1);
    assert_eq!(found.notes.len(), 1);
    assert!(found.devices[0].lighting().is_none());
    assert!(found.devices[0].battery().is_some());
    assert!(
        !found.devices[0]
            .descriptor()
            .details
            .iter()
            .any(|d| matches!(d, Detail::Battery(_)))
    );
    assert_eq!(fake.log().len(), 1);
}

#[test]
fn radio_telemetry_recovers_without_removing_or_reidentifying_the_device() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let awake = Arc::new(AtomicBool::new(false));
    let responding = Arc::clone(&awake);
    let fake = FakeBus::new(
        vec![endpoint()],
        Box::new(move |packet| {
            assert_eq!(packet, &Packet::Output(radio_codec::request().to_vec()));
            if responding.load(Ordering::Relaxed) {
                vec![captured()]
            } else {
                Vec::new()
            }
        }),
    );
    fake.set_descriptor(radio_codec::DESCRIPTOR.to_vec());
    let bus: Arc<dyn Bus> = fake.clone();
    let mut found = ByTechDriver.discover(&bus);
    assert_eq!(found.devices.len(), 1);
    assert_eq!(found.notes.len(), 1);
    assert_eq!(fake.log().len(), 1);
    let device = &mut found.devices[0];
    assert!(device.lighting().is_none());
    assert!(
        !device
            .descriptor()
            .details
            .iter()
            .any(|d| matches!(d, Detail::Battery(_)))
    );
    fake.set_descriptor(vec![0]);
    awake.store(true, Ordering::Relaxed);
    device.battery().unwrap().refresh_battery().unwrap();
    assert!(device.descriptor().details.contains(&Detail::Battery(77)));
    assert_eq!(fake.log().len(), 2);
    assert!(fake.backups().is_empty());
}

#[test]
fn malformed_telemetry_is_reported_without_losing_the_confirmed_receiver() {
    let mut bad = captured();
    bad[19] ^= 1;
    let fake = bus(bad);
    let shared: Arc<dyn Bus> = fake.clone();
    let mut found = ByTechDriver.discover(&shared);
    assert_eq!(found.devices.len(), 1);
    assert_eq!(found.notes.len(), 1);
    assert!(found.devices[0].lighting().is_none());
    assert!(found.devices[0].battery().is_some());
    assert!(
        !found.devices[0]
            .descriptor()
            .details
            .iter()
            .any(|d| matches!(d, Detail::Battery(_)))
    );
    assert_eq!(fake.log().len(), 1);
}
#[test]
fn radio_rejects_foreign_endpoints_and_survives_failed_usb_confirmation() {
    assert!(radio::admits(&endpoint()));
    for (vid, pid) in [
        (0x3554, 0xf52d),
        (0x291d, 0x385d),
        (0x258a, 0x010c),
        (0x0c45, 0x8009),
    ] {
        let mut e = endpoint();
        e.vid = vid;
        e.pid = pid;
        assert!(!radio::admits(&e));
    }
    let mut e = endpoint();
    e.release = 0;
    assert!(!radio::admits(&e));
    let wired = Endpoint::hid(0x258a, 0x010c, 0xff00, 1).with_reports(Some(ReportSizes {
        input: 8,
        output: 0,
        feature: 520,
    }));
    let fake = FakeBus::new(
        vec![endpoint(), wired],
        Box::new(|packet| {
            if matches!(packet, Packet::Output(_)) {
                vec![captured()]
            } else {
                Vec::new()
            }
        }),
    );
    fake.set_descriptor(radio_codec::DESCRIPTOR.to_vec());
    let shared: Arc<dyn Bus> = fake.clone();
    let found = ByTechDriver.discover(&shared);
    assert_eq!(found.devices.len(), 1);
    assert!(
        found.devices[0]
            .descriptor()
            .details
            .contains(&Detail::Battery(77))
    );
    assert!(fake.log().iter().any(|p| matches!(p, Packet::Output(_))));
}

#[test]
fn live_radio_reads_do_not_repeat_descriptor_discovery() {
    let fake = bus(captured());
    let shared: Arc<dyn Bus> = fake.clone();
    let mut device = ByTechDriver.discover(&shared).devices.remove(0);
    fake.set_descriptor(vec![0]);
    device.battery().unwrap().refresh_battery().unwrap();
    assert_eq!(fake.log().len(), 2);
    assert!(
        fake.log()
            .iter()
            .all(|p| p == &Packet::Output(radio_codec::request().to_vec()))
    );
}
