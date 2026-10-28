use super::digest::sha256_hex;
use serde::{Deserialize, Serialize};

/// Input / Output / Feature report sizes in bytes, including the Report ID byte, as Windows
/// reports them in `HIDP_CAPS`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportSizes {
    pub input: u16,
    pub output: u16,
    pub feature: u16,
}

/// One HID top-level collection as seen during enumeration. Passive data only: admission checks
/// run on it without opening the device. The OS path is private and never serialized; outside
/// code sees only its hash.
#[derive(Clone, Debug)]
pub struct Endpoint {
    pub vid: u16,
    pub pid: u16,
    pub usage_page: u16,
    pub usage: u16,
    pub interface: i32,
    pub release: u16,
    pub product: String,
    pub reports: Option<ReportSizes>,
    path: Vec<u8>,
}

impl Endpoint {
    pub fn hid(vid: u16, pid: u16, usage_page: u16, usage: u16) -> Self {
        Self {
            vid,
            pid,
            usage_page,
            usage,
            interface: -1,
            release: 0,
            product: String::new(),
            reports: None,
            path: Vec::new(),
        }
    }

    pub fn with_interface(mut self, interface: i32) -> Self {
        self.interface = interface;
        self
    }

    pub fn with_release(mut self, release: u16) -> Self {
        self.release = release;
        self
    }

    pub fn with_product(mut self, product: impl Into<String>) -> Self {
        self.product = product.into();
        self
    }

    pub fn with_reports(mut self, reports: Option<ReportSizes>) -> Self {
        self.reports = reports;
        self
    }

    pub fn with_path(mut self, path: Vec<u8>) -> Self {
        self.path = path;
        self
    }

    pub(crate) fn path(&self) -> &[u8] {
        &self.path
    }

    /// Stable per-port fingerprint of the endpoint. Safe to store: the path cannot be recovered.
    pub fn fingerprint(&self) -> String {
        sha256_hex(&self.path)
    }

    pub fn has_reports(&self, input: u16, output: u16) -> bool {
        self.reports
            .is_some_and(|r| r.input == input && r.output == output)
    }

    pub fn has_feature(&self, feature: u16) -> bool {
        self.reports.is_some_and(|r| r.feature == feature)
    }
}
