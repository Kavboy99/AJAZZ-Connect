use crate::core::Endpoint;
use serde::Deserialize;

/// A passive match over identifiers and report sizes. Every field that is set must match.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    vid: Option<u16>,
    #[serde(default)]
    vids: Vec<u16>,
    pid: Option<u16>,
    usage_page: Option<u16>,
    usage: Option<u16>,
    input_bytes: Option<u16>,
    output_bytes: Option<u16>,
    feature_bytes: Option<u16>,
    product_contains: Option<String>,
}

impl Rule {
    pub fn matches(&self, e: &Endpoint) -> bool {
        let sizes = e.reports;
        let size = |want: Option<u16>, get: fn(&crate::core::ReportSizes) -> u16| {
            want.is_none_or(|w| sizes.as_ref().is_some_and(|s| get(s) == w))
        };
        self.vid.is_none_or(|v| v == e.vid)
            && (self.vids.is_empty() || self.vids.contains(&e.vid))
            && self.pid.is_none_or(|p| p == e.pid)
            && self.usage_page.is_none_or(|p| p == e.usage_page)
            && self.usage.is_none_or(|u| u == e.usage)
            && size(self.input_bytes, |s| s.input)
            && size(self.output_bytes, |s| s.output)
            && size(self.feature_bytes, |s| s.feature)
            && self
                .product_contains
                .as_deref()
                .is_none_or(|part| e.product.to_ascii_uppercase().contains(part))
    }
}
