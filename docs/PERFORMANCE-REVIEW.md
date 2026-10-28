# Battery polling performance review

Reviewed: 2026-10-28. Scope: current source, installed egui/eframe 0.33.3 implementation,
USB/radio battery read paths, session scheduling and current settings. No application code changed.

## Findings

1. **P2: foreground polling is unrelated to whether battery data is visible.**
   `src/ui/app.rs:140` schedules telemetry whenever any battery-capable device exists.
   `src/app/session/battery.rs:12` selects every battery-capable device. Thus Settings,
   Diagnostics and device lists without a battery indicator still cause reads every five seconds.
   One device means up to 720 cycles/hour. Restrict fast polling to a visible battery surface;
   use a slower background policy elsewhere.

2. **P2: minimized state is not a complete background/tray model.**
   `src/ui/app/battery.rs:20` reads only `ViewportInfo::minimized`. There is no native tray
   integration or hidden-window lifecycle in the application source. An unfocused window
   remains on the five-second schedule. A future tray implementation cannot safely assume
   that hiding the window sets the minimized flag. Model visibility/focus and an explicit
   hidden/tray state separately; schedule an immediate refresh on restoration.
   Installed egui 0.33.3 source was checked; newer-version viewport fields must not be assumed.

3. **P2: background reads currently provide no implemented background feature.**
   Five minutes is only 12 cycles/hour per device, not evidence of excessive CPU usage.
   Nevertheless there are no tray battery display or low-battery notifications to consume
   background readings. Each cycle opens HID, sends commands, starts one short-lived thread,
   and wakes the GUI on completion even if values did not change. Radio interactions may
   influence peripheral sleep; this has not been measured on the owner's keyboard.
   Default background polling off is reasonable until a background feature needs it.

4. **P2: slow devices and failure backoff are global.**
   `src/app/session/battery.rs:24` reads devices sequentially in one worker and returns all
   results together. A radio timeout can wait about 2.8 seconds per unresponsive receiver.
   While devices are borrowed by the worker, `Session::busy` also postpones lighting/discovery.
   `src/ui/app.rs:99` backs off the whole schedule if any device fails. Prefer independent
   deadlines/backoff and distinguish telemetry ownership from user-action status as support grows.

## Existing strengths

- No permanent worker pool, busy loop or sleeping background thread.
- HID handles close after each operation; no repeated global device discovery.
- Radio descriptors are validated on discovery, not on each refresh.
- No overlapping telemetry/discovery/lighting writes; radio receives are bounded, without resend.
- No battery capability means no telemetry deadline wakeups.
- Restore refresh and failure backoff have hardware-free scheduling tests.

## Recommended product policy

Default to **Automatic** rather than expose a raw millisecond interval.

| State | Recommended starting policy |
| --- | --- |
| Battery indicator visible, on battery | Every 30 seconds |
| Battery indicator visible, external power below full | Every 5-10 seconds |
| Window open on a page without battery information | Pause, or every 60 seconds if needed |
| Minimized/hidden, no background notifications | Pause; refresh on restore |
| Minimized/hidden, background notifications enabled in a future version | Every 30 minutes; optionally faster on a previously observed low charge |
| Unresponsive device | Per-device backoff; avoid repeatedly waking a sleeping receiver |

Optional presets can be Automatic, Responsive and Power saving. A separate advanced background
setting can offer Off, 15, 30 or 60 minutes. These are design recommendations, not measured optimal
hardware intervals. Shorter background intervals trade timely low-battery notifications for wakeups.
Changing the interval must not trigger rediscovery or persistent hardware writes.

## Measurement limits and follow-up

No matching running application process was available during the attempted 15-second CPU/memory
sample, so this review reports no runtime CPU or memory figures. Earlier offscreen `--review`
scenes intentionally disable telemetry and do not establish minimized/tray performance.

A useful live benchmark compares the same build with polling off, every five minutes and every
30 minutes, under verified window states and with the receiver idle. Measure cumulative CPU time,
working set, transient thread creation, HID request counts and actual device sleep behavior.
Use a long enough interval to observe background deadlines, without logging keys or unique IDs.
Microsoft documents USB selective suspend and ETW-based validation:
https://learn.microsoft.com/en-us/windows-hardware/drivers/usbcon/usb-etw-and-power-management
This is general power-management evidence, not proof that this keyboard is awakened by this query.

## Clarified product requirement: tray battery display

The owner clarified that battery levels will be displayed in the tray, without Windows notifications,
and many wireless devices may be connected. This supersedes the earlier recommendation to disable
background polling when notifications are absent. Tray battery display is itself a background
consumer of readings. Tray UI implementation is still future work, not an existing application feature.

Recommended starting policy: five-minute background refresh per battery-powered device, one-minute
refresh while its observed state indicates charging, and immediate refresh when the main window is
restored. Visible device details can use a shorter interval. These are starting defaults to benchmark,
not measured optimal intervals. For example, 20 devices at five-minute intervals produce 240 logical
refreshes/hour, averaging four per minute; protocol transaction count depends on the driver.

Use one scheduler with per-device deadlines, independent error backoff, bounded queued work and
serialization for devices sharing a receiver. Stagger or coalesce due reads to avoid a simultaneous
startup/interval burst. Do not create one permanent timer or idle worker per device, re-enumerate USB
for each sample, or delay all devices because one receiver sleeps. Each device's last successful
reading and freshness must be tracked separately; stale data must not be presented as a fresh sample.
Update tray imagery only when its displayed value or state changes, without repainting the hidden
main window just to show unchanged telemetry. Suspend work for removed/disabled devices and prefer
passive battery events when a future driver actually supports them.

Expose an optional global preset (Automatic / Responsive / Power saving) rather than require separate
frequency configuration for every peripheral. Protocol-specific minimums, timeout behavior and power
constraints belong to drivers. Reassess the five-minute default with real multi-device measurements;
notification preferences must not determine whether tray battery readings refresh.

## Implemented follow-up

The global five-second/all-device schedule and global error backoff have been replaced by per-device
clocks and a one-job dispatcher. Foreground scope is the currently open device card; other devices
and pages use slower intervals. Automatic / Responsive / Eco presets are persisted in Settings.
Background polling remains enabled independently of notifications, with a shorter interval during
observed charging. Each device retains its last successful sample and timestamp for future tray use.
Hidden/minimized frames skip shell/model/page construction, though the event loop still wakes for
scheduled work and completion. An explicit hidden-window hook is available for future tray integration.
Actual native tray icons and a multi-device runtime CPU/power benchmark remain outside this change.
