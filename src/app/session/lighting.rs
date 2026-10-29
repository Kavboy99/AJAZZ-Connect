//! Latest-value lighting queue. Only user edits enqueue; rendering and discovery never do.
use super::{Events, Session};
use crate::core::LightingSettings;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

const WRITE_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Default)]
pub(super) struct LiveLighting {
    pending: BTreeMap<usize, LightingSettings>,
    sent: BTreeMap<usize, (LightingSettings, Instant)>,
}
impl LiveLighting {
    fn enqueue(&mut self, slot: usize, settings: LightingSettings) {
        if self
            .sent
            .get(&slot)
            .is_some_and(|(last, _)| *last == settings)
        {
            self.pending.remove(&slot);
        } else {
            self.pending.insert(slot, settings);
        }
    }
    pub(super) fn cancel(&mut self, slot: usize) {
        self.pending.remove(&slot);
        self.sent.remove(&slot);
    }
    pub(super) fn pending(&self) -> bool {
        !self.pending.is_empty()
    }
}

impl Session {
    pub fn queue_lighting(&mut self, slot: usize, settings: LightingSettings) {
        if self.entries.get(slot).is_some_and(|e| {
            !e.dormant
                && e.lighting.is_some_and(|l| l.writable && !l.persistent)
                && e.editor.is_some()
        }) {
            self.live_lighting.enqueue(slot, settings);
        }
    }
    /// A pending deadline needs one wakeup; busy workers already wake the app on completion.
    pub fn flush_lighting(&mut self, events: &Events) -> Option<Duration> {
        self.flush_explicit_lighting(events);
        let now = Instant::now();
        let slots: Vec<_> = self.live_lighting.pending.keys().copied().collect();
        let mut wake: Option<Duration> = None;
        for slot in slots {
            let Some(entry) = self.entries.get(slot) else {
                self.live_lighting.cancel(slot);
                continue;
            };
            if entry.busy() {
                continue;
            }
            if let Some((_, last)) = self.live_lighting.sent.get(&slot) {
                let wait = WRITE_INTERVAL.saturating_sub(now.saturating_duration_since(*last));
                if !wait.is_zero() {
                    wake = Some(wake.map_or(wait, |w| w.min(wait)));
                    continue;
                }
            }
            if let Some(settings) = self.live_lighting.pending.remove(&slot) {
                self.live_lighting.sent.insert(slot, (settings, now));
                self.apply_lighting(slot, settings, events);
            }
        }
        wake
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn settings(speed: u8) -> LightingSettings {
        LightingSettings {
            preset: 1,
            brightness: 3,
            speed,
            color: None,
        }
    }
    #[test]
    fn rapid_edits_keep_only_the_latest_and_returning_to_inflight_cancels_pending() {
        let mut queue = LiveLighting::default();
        queue.sent.insert(0, (settings(1), Instant::now()));
        for speed in [2, 3, 4, 0] {
            queue.enqueue(0, settings(speed));
        }
        assert_eq!(queue.pending.get(&0), Some(&settings(0)));
        queue.enqueue(0, settings(1));
        assert!(!queue.pending());
    }
    #[test]
    fn failure_or_restore_cancels_queued_writes_and_allows_an_explicit_new_edit() {
        let mut queue = LiveLighting::default();
        queue.enqueue(0, settings(2));
        queue.cancel(0);
        assert!(!queue.pending());
        queue.enqueue(0, settings(2));
        assert!(queue.pending());
    }
}

#[cfg(test)]
mod integration_tests;
