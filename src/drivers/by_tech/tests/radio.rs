use super::*;

#[test]
fn confirmed_wired_device_still_takes_precedence_over_the_receiver() {
    let (_, state) = keyboard(true);
    let mut wired = responder(state);
    let radio = Endpoint::hid(0x3554, 0xfa09, 0xff02, 2)
        .with_release(0x2008)
        .with_reports(Some(ReportSizes {
            input: 20,
            output: 20,
            feature: 0,
        }));
    let fake = FakeBus::new(
        vec![endpoint(), radio],
        Box::new(move |packet| {
            assert!(
                matches!(packet, Packet::Feature(_)),
                "confirmed USB must avoid duplicate radio discovery"
            );
            wired(packet)
        }),
    );
    let bus: Arc<dyn Bus> = fake.clone();
    let mut found = ByTechDriver.discover(&bus);
    assert_eq!(found.devices.len(), 1);
    assert!(found.devices[0].lighting().is_some());
    assert!(found.notes.is_empty());
}
