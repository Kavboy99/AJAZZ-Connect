use crate::core::{Detail, PowerSource};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatteryMode {
    #[default]
    Automatic,
    Responsive,
    Eco,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatteryView {
    Background,
    Window,
    Device(usize),
}

impl BatteryMode {
    pub fn interval(self, view: BatteryView, slot: usize, details: &[Detail]) -> Duration {
        let charging = details.contains(&Detail::Power(PowerSource::External))
            && details
                .iter()
                .any(|d| matches!(d,Detail::Battery(p) if *p<100));
        let seconds = match (self, view) {
            (Self::Automatic, BatteryView::Device(s)) if s == slot => 5,
            (Self::Responsive, BatteryView::Device(s)) if s == slot => 2,
            (Self::Eco, BatteryView::Device(s)) if s == slot => 15,
            (Self::Automatic, BatteryView::Background) => {
                if charging {
                    60
                } else {
                    300
                }
            }
            (Self::Responsive, BatteryView::Background) => {
                if charging {
                    30
                } else {
                    120
                }
            }
            (Self::Eco, BatteryView::Background) => {
                if charging {
                    300
                } else {
                    900
                }
            }
            (Self::Automatic, _) => 60,
            (Self::Responsive, _) => 30,
            (Self::Eco, _) => 120,
        };
        Duration::from_secs(seconds)
    }
}

pub(crate) struct BatteryClock {
    pub last_reading: Vec<Detail>,
    pub due: Instant,
    pub in_flight: bool,
    pub updated: Option<Instant>,
    interval: Duration,
    failures: u32,
}
impl BatteryClock {
    pub fn new() -> Self {
        Self {
            last_reading: Vec::new(),
            due: Instant::now() + Duration::from_secs(5),
            in_flight: false,
            updated: None,
            interval: Duration::ZERO,
            failures: 0,
        }
    }
    pub fn configure(&mut self, now: Instant, interval: Duration, force: bool, slot: usize) {
        let stagger = Duration::from_millis((slot as u64 % 20) * 100);
        if force {
            self.due = now + stagger;
        } else if self.interval != interval && self.failures == 0 {
            self.due = self.updated.unwrap_or(now) + interval + stagger;
        }
        self.interval = interval;
    }
    pub fn complete(&mut self, now: Instant, success: bool) {
        self.in_flight = false;
        if success {
            self.updated = Some(now);
            self.failures = 0;
            self.due = now + self.interval;
        } else {
            self.failures = self.failures.saturating_add(1);
            self.due = now
                + Duration::from_secs(
                    (60u64.saturating_mul(1u64 << self.failures.min(5).saturating_sub(1))).min(900),
                )
                .max(self.interval);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn policy_matches_device_scope_and_power() {
        let battery = [Detail::Battery(50), Detail::Power(PowerSource::Battery)];
        let charging = [Detail::Battery(50), Detail::Power(PowerSource::External)];
        assert_eq!(
            BatteryMode::Automatic
                .interval(BatteryView::Background, 0, &battery)
                .as_secs(),
            300
        );
        assert_eq!(
            BatteryMode::Automatic
                .interval(BatteryView::Background, 0, &charging)
                .as_secs(),
            60
        );
        assert_eq!(
            BatteryMode::Automatic
                .interval(BatteryView::Device(1), 0, &battery)
                .as_secs(),
            60
        );
        assert_eq!(
            BatteryMode::Automatic
                .interval(BatteryView::Device(1), 1, &battery)
                .as_secs(),
            5
        );
        for mode in [
            BatteryMode::Automatic,
            BatteryMode::Responsive,
            BatteryMode::Eco,
        ] {
            assert!(
                mode.interval(BatteryView::Background, 0, &battery)
                    >= mode.interval(BatteryView::Window, 0, &battery)
            );
        }
    }
    #[test]
    fn clocks_have_independent_backoff_and_stagger() {
        let now = Instant::now();
        let mut first = BatteryClock::new();
        let mut second = BatteryClock::new();
        first.configure(now, Duration::from_secs(5), true, 0);
        second.configure(now, Duration::from_secs(5), true, 1);
        assert_eq!(
            second.due.duration_since(first.due),
            Duration::from_millis(100)
        );
        first.complete(now, false);
        second.complete(now, true);
        assert_eq!(first.due.duration_since(now), Duration::from_secs(60));
        assert_eq!(second.due.duration_since(now), Duration::from_secs(5));
        first.configure(now, Duration::from_secs(2), false, 0);
        assert_eq!(first.due.duration_since(now), Duration::from_secs(60));
        second.configure(now, Duration::from_secs(300), false, 1);
        second.configure(now, Duration::from_secs(5), true, 1);
        assert_eq!(second.due.duration_since(now), Duration::from_millis(100));
    }
    #[test]
    fn legacy_settings_and_presets_round_trip() {
        let old: super::super::Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(old.battery_mode, BatteryMode::Automatic);
        for mode in [
            BatteryMode::Automatic,
            BatteryMode::Responsive,
            BatteryMode::Eco,
        ] {
            let settings = super::super::Settings {
                battery_mode: mode,
                ..Default::default()
            };
            let round: super::super::Settings =
                serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
            assert_eq!(round.battery_mode, mode);
        }
    }
}
