use super::{Interface, Report};
use crate::classify;
use crate::core::{Bus, Driver, Endpoint};

const DESCRIPTOR_LIMIT: usize = 16384;

/// Vendor pages and LampArray carry the protocol-specific reports worth recording.
fn wants_descriptor(endpoint: &Endpoint) -> bool {
    endpoint.usage_page >= 0xff00 || endpoint.usage_page == 0x59
}

/// The descriptor as Windows reconstructs it from preparsed data. No report is exchanged.
fn read_descriptor(bus: &dyn Bus, endpoint: &Endpoint) -> Option<Vec<u8>> {
    let channel = bus.open_hid(endpoint).ok()?;
    let mut buffer = vec![0; DESCRIPTOR_LIMIT];
    let n = channel.report_descriptor(&mut buffer).ok()?;
    (n < buffer.len()).then(|| buffer[..n].to_vec())
}

/// Passive report of the interfaces of interest. The only requests sent are the drivers' own
/// read-only identification requests, and only to endpoints those drivers admit.
pub fn probe(bus: &dyn Bus, drivers: &[Box<dyn Driver>]) -> Report {
    let mut report = Report::empty();
    for endpoint in bus
        .hid_endpoints()
        .iter()
        .filter(|e| classify::is_of_interest(e))
    {
        let mut interface = Interface::of(endpoint);
        if endpoint.reports.is_none() {
            interface.read_error = Some("HID capabilities unavailable".into());
        }
        if wants_descriptor(endpoint)
            && let Some(descriptor) = read_descriptor(bus, endpoint)
        {
            interface.report_descriptor_sha256 = Some(crate::core::sha256_hex(&descriptor));
            interface.report_descriptor_hex =
                Some(descriptor.iter().map(|b| format!("{b:02x}")).collect());
        }
        match drivers.iter().find_map(|d| d.identify(bus, endpoint)) {
            Some(Ok(identification)) => interface.identification = Some(identification),
            Some(Err(error)) => interface.read_error = Some(error.to_string()),
            None => {}
        }
        report.interfaces.push(interface);
    }
    report
}
