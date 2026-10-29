//! An explicit click waits for telemetry to release the device without turning into edit streaming.
use super::{Events, Session, Status};
use crate::core::LightingSettings;

impl Session {
    pub fn commit_lighting(
        &mut self,
        slot: usize,
        settings: LightingSettings,
        events: &Events,
    ) -> bool {
        let Some(entry) = self.entries.get_mut(slot) else {
            return false;
        };
        if entry.is_dormant()
            || !entry.lighting.is_some_and(|t| t.writable)
            || entry.editor.is_none()
            || entry.lighting_pending()
            || entry.status == Some(Status::Running)
        {
            return false;
        }
        if entry.busy() {
            entry.pending_lighting = Some(settings);
        } else {
            self.apply_lighting(slot, settings, events);
        }
        true
    }

    pub(super) fn flush_explicit_lighting(&mut self, events: &Events) {
        let ready: Vec<_> = self
            .entries
            .iter_mut()
            .enumerate()
            .filter_map(|(slot, e)| {
                if e.busy() {
                    return None;
                }
                e.pending_lighting.take().map(|settings| (slot, settings))
            })
            .collect();
        for (slot, settings) in ready {
            self.apply_lighting(slot, settings, events);
        }
    }
}
