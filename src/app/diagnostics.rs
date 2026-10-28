use super::events::{Event, Events};
use crate::core::DriverError;
use crate::diagnostics::{self, Report};
use crate::drivers;
use crate::storage::atomic;
use crate::transport::SystemBus;
use std::path::PathBuf;

/// State of the diagnostics page: the last passive report.
#[derive(Default)]
pub struct Diagnostics {
    report: Option<Result<Report, DriverError>>,
    saved: Option<Result<PathBuf, String>>,
    probing: bool,
}

impl Diagnostics {
    pub fn report(&self) -> Option<&Result<Report, DriverError>> {
        self.report.as_ref()
    }

    pub fn saved(&self) -> Option<&Result<PathBuf, String>> {
        self.saved.as_ref()
    }

    pub fn probing(&self) -> bool {
        self.probing
    }

    pub fn probe(&mut self, events: &Events) {
        if self.probing {
            return;
        }
        self.probing = true;
        self.saved = None;
        events.spawn(|| {
            let bus = SystemBus::new();
            Event::Probed(Box::new(match bus.hid_error() {
                Some(error) => Err(error.clone()),
                None => Ok(diagnostics::probe(&bus, &drivers::registry())),
            }))
        });
    }

    pub fn finish_probe(&mut self, result: Result<Report, DriverError>) {
        self.probing = false;
        self.report = Some(result);
    }

    pub fn save_report(&mut self, path: PathBuf) {
        let Some(Ok(report)) = &self.report else {
            return;
        };
        self.saved = Some(
            atomic::write_json(&path, report)
                .map(|()| path)
                .map_err(|e| e.to_string()),
        );
    }
}
