use super::*;
use crate::core::Outcome;

fn descriptor() -> Vec<u8> {
    let hex = "0600ff0901a10185060902150026ff0075089507810019012902150026ff007508960702b102c0";
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn capture_and_recover_corrupt_configuration_preserves_backup_and_never_writes_palette() {
    let (fake, state) = keyboard(true);
    fake.set_descriptor(descriptor());
    let bus: Arc<dyn Bus> = fake.clone();
    let mut device = ByTechDriver.discover(&bus).devices.remove(0);
    let start = fake.log().len();
    let snapshot = device.configuration_backup().unwrap().capture().unwrap();
    assert!(
        opcodes(&fake.log()[start..])
            .iter()
            .all(|op| op & 0x80 != 0)
    );
    assert!(fake.backups().is_empty());
    let packed = crate::storage::archive::encode(&snapshot).unwrap();
    let snapshot = crate::storage::archive::decode(&packed).unwrap();
    {
        let mut k = state.lock().unwrap();
        k.config[0] = 255;
        k.config[10] = 255;
        k.colors[21] = 88;
    }
    // Recovery must be discoverable even when the current editor cannot parse its settings.
    let mut device = ByTechDriver.discover(&bus).devices.remove(0);
    assert!(device.lighting().unwrap().editor().is_none());
    let start = fake.log().len();
    assert_eq!(
        device.configuration_backup().unwrap().recover(&snapshot),
        Ok(Outcome::ConfigurationRestored)
    );
    let log = fake.log();
    let writes: Vec<_> = opcodes(&log[start..])
        .into_iter()
        .filter(|op| op & 0x80 == 0)
        .collect();
    assert_eq!(writes, [4]);
    let first_write = log
        .iter()
        .position(|p| matches!(p, Packet::Feature(p) if p[1] == 4))
        .unwrap();
    assert!(fake.packets_before_backup().unwrap() <= first_write);
    let before: serde_json::Value = serde_json::from_slice(&fake.backups()[0].1).unwrap();
    assert_eq!(before["config"][10], 255);
    let k = state.lock().unwrap();
    assert_eq!(k.config, config());
    assert_eq!(
        k.colors[21], 88,
        "palette remains unchanged during recovery"
    );
}

#[test]
fn foreign_identity_or_descriptor_never_causes_recovery_write() {
    let (fake, _) = keyboard(true);
    let bus: Arc<dyn Bus> = fake.clone();
    let mut device = ByTechDriver.discover(&bus).devices.remove(0);
    let snapshot = device.configuration_backup().unwrap().capture().unwrap();
    let mut value: serde_json::Value = snapshot.parse().unwrap();
    value["device_identity"] = "foreign".into();
    let foreign = Snapshot::from_bytes(serde_json::to_vec(&value).unwrap()).unwrap();
    let start = fake.log().len();
    assert_eq!(
        device.configuration_backup().unwrap().recover(&foreign),
        Err(DriverError::ForeignBackup)
    );
    assert_eq!(fake.log().len(), start);
    fake.set_descriptor(vec![0; 39]);
    assert_eq!(
        device.configuration_backup().unwrap().recover(&snapshot),
        Err(DriverError::Changed)
    );
    assert!(
        opcodes(&fake.log()[start..])
            .iter()
            .all(|op| op & 0x80 != 0)
    );
    assert!(fake.backups().is_empty());
}

#[test]
fn failed_recovery_readback_is_not_retried() {
    let (fake, state) = keyboard(false);
    fake.set_descriptor(descriptor());
    let bus: Arc<dyn Bus> = fake.clone();
    let mut device = ByTechDriver.discover(&bus).devices.remove(0);
    let snapshot = device.configuration_backup().unwrap().capture().unwrap();
    state.lock().unwrap().config[0] = 99;
    assert_eq!(
        device.configuration_backup().unwrap().recover(&snapshot),
        Err(DriverError::ReadBackMismatch)
    );
    assert_eq!(
        opcodes(&fake.log()).iter().filter(|&&op| op == 4).count(),
        1
    );
    assert_eq!(fake.backups().len(), 1);
}

#[test]
fn palette_write_framing_and_mismatched_full_headers_are_checked() {
    let packet = codec::write(Command::WriteColors, &colors()).unwrap();
    assert_eq!(&packet[..8], &[6, 10, 0, 0, 1, 0, 0, 2]);
    assert_eq!(&packet[8..], colors());
    for command in [
        Command::ReadPassword,
        Command::ReadConfig,
        Command::ReadColors,
    ] {
        let packet = codec::request(command).unwrap();
        let mut report = packet[..8].to_vec();
        report.resize(8 + command.length(), 0);
        for index in 2..8 {
            let mut wrong = report.clone();
            wrong[index] ^= 1;
            assert!(
                codec::reply(&wrong, command).is_err(),
                "header byte {index}"
            );
        }
    }
}

#[test]
fn failed_pre_recovery_backup_prevents_any_write() {
    let (fake, state) = keyboard(true);
    fake.set_descriptor(descriptor());
    let bus: Arc<dyn Bus> = fake.clone();
    let mut device = ByTechDriver.discover(&bus).devices.remove(0);
    let snapshot = device.configuration_backup().unwrap().capture().unwrap();
    state.lock().unwrap().config[0] = 99;
    fake.fail_backups();
    assert!(matches!(
        device.configuration_backup().unwrap().recover(&snapshot),
        Err(DriverError::Io(_))
    ));
    assert!(opcodes(&fake.log()).iter().all(|op| op & 0x80 != 0));
    assert_eq!(state.lock().unwrap().config[0], 99);
}
