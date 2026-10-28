//! Passive classification of HID interfaces for diagnostics. A family match is a hypothesis about
//! the controller, not a compatibility proof.

use crate::core::Endpoint;
use crate::profiles::{self, Family};

/// The family that explains `endpoint`, first match in priority order.
pub fn classify(endpoint: &Endpoint) -> Option<&'static Family> {
    profiles::families().iter().find(|f| f.matches(endpoint))
}

/// Whether the diagnostic probe should record `endpoint`. Unknown devices of other vendors are
/// left out of reports.
pub fn is_of_interest(endpoint: &Endpoint) -> bool {
    classify(endpoint).is_some() || profiles::describe(endpoint).is_some()
}
