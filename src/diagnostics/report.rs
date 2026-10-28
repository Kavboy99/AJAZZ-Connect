use crate::app::identity;
use crate::classify;
use crate::core::{Endpoint, Identification, ReportSizes};
use crate::firmware::PackageInfo;
use crate::profiles::Family;
use serde::{Deserialize, Serialize};

const SCHEMA: u32 = 2;
const NOTICE: &str = "Passive report: descriptors and identification replies only. \
    No firmware, settings or bootloader requests were sent. Interface IDs are hashes; no unique \
    hardware identifiers or OS paths are included.";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Interface {
    /// First 16 hex digits of the hashed device path.
    pub id: String,
    pub vid: u16,
    pub pid: u16,
    pub product: String,
    pub usage_page: u16,
    pub usage: u16,
    pub interface: i32,
    pub input_bytes: Option<u16>,
    pub output_bytes: Option<u16>,
    pub feature_bytes: Option<u16>,
    #[serde(default)]
    pub identification: Option<Identification>,
    pub read_error: Option<String>,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub protocol_family: String,
    #[serde(default)]
    pub control_hint: String,
    #[serde(default)]
    pub bcd_device: u16,
    #[serde(default)]
    pub report_descriptor_hex: Option<String>,
    #[serde(default)]
    pub report_descriptor_sha256: Option<String>,
}

impl Interface {
    pub fn of(endpoint: &Endpoint) -> Self {
        let family = classify::classify(endpoint);
        let fingerprint = endpoint.fingerprint();
        Self {
            id: fingerprint[..16].to_owned(),
            vid: endpoint.vid,
            pid: endpoint.pid,
            product: endpoint.product.clone(),
            usage_page: endpoint.usage_page,
            usage: endpoint.usage,
            interface: endpoint.interface,
            input_bytes: endpoint.reports.map(|r| r.input),
            output_bytes: endpoint.reports.map(|r| r.output),
            feature_bytes: endpoint.reports.map(|r| r.feature),
            identification: None,
            read_error: None,
            profile: crate::profiles::describe(endpoint)
                .map(|p| format!("{} {}", p.vendor, p.model)),
            protocol_family: family.map_or_else(String::new, |f| f.name.clone()),
            control_hint: family.map_or_else(String::new, |f| f.hint.en.clone()),
            bcd_device: endpoint.release,
            report_descriptor_hex: None,
            report_descriptor_sha256: None,
        }
    }

    /// The passive endpoint this record describes, for admission checks on saved reports.
    pub fn endpoint(&self) -> Endpoint {
        let reports = match (self.input_bytes, self.output_bytes, self.feature_bytes) {
            (Some(input), Some(output), Some(feature)) => Some(ReportSizes {
                input,
                output,
                feature,
            }),
            _ => None,
        };
        Endpoint::hid(self.vid, self.pid, self.usage_page, self.usage)
            .with_interface(self.interface)
            .with_release(self.bcd_device)
            .with_product(self.product.clone())
            .with_reports(reports)
    }

    pub fn family(&self) -> Option<&'static Family> {
        classify::classify(&self.endpoint())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub schema: u32,
    pub application_version: String,
    pub created_utc: String,
    pub interfaces: Vec<Interface>,
    #[serde(default)]
    pub firmware: Option<PackageInfo>,
    pub firmware_writes: bool,
    pub settings_writes: bool,
    pub boot_requests: bool,
    #[serde(default)]
    pub notice: String,
}

impl Report {
    pub fn empty() -> Self {
        Self {
            schema: SCHEMA,
            application_version: identity::version().into(),
            created_utc: chrono::Utc::now().to_rfc3339(),
            interfaces: Vec::new(),
            firmware: None,
            firmware_writes: false,
            settings_writes: false,
            boot_requests: false,
            notice: NOTICE.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Report;

    #[test]
    fn report_never_claims_writes_and_carries_no_private_fields() {
        let report = Report::empty();
        assert!(!report.firmware_writes && !report.settings_writes && !report.boot_requests);
        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains("serial") && !json.contains("device_path"));
    }
}
