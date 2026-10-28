# Architecture

This document describes how the code is organized (revision 0.3.0) and what is still a **target**.
Anything marked "target" is not implemented yet. Do not present the target as existing.

**Direction.** The product uses a proprietary, project-owned device-control system with direct protocol adapters for explicitly supported hardware revisions. Current development targets are the SONiX and BY Tech AK820 MAX variants. The core and UI remain capability-based; hardware-specific commands stay in drivers and profiles. No external RGB service is part of the product architecture.

## 1. Layers

One crate, modules by responsibility. A codec does not know about HID, a driver does not know about the
UI, the UI does not know about specific drivers.

```
src/
  main.rs                  entry point only: cli::run or ui::run
  lib.rs                   module list

  core/                    shared types and contracts; no IO and no brand names
    bus.rs                 trait Bus (the driver's only door to IO), HidChannel, Stream, Guard
    driver.rs              trait Driver, trait Device, DriverInfo, Maturity, Discovery, Identification
    capability.rs          trait Lighting, LightingTraits, Rollback, Outcome
    device.rs              DeviceDescriptor, DeviceKind, Detail
    endpoint.rs            Endpoint (passive HID collection; private path), ReportSizes
    error.rs               DriverError (typed; English details)
    snapshot.rs            Snapshot: a size-limited backup file for a driver to validate
    color.rs, digest.rs    Rgb; SHA256 of private identifiers

  transport/               the only place with unsafe and real IO
    system_bus.rs          SystemBus: impl Bus over one HID enumeration, loopback TCP, locks, backups
    hid.rs                 hidapi wrapper: endpoints, short-lived channels
    hid_caps.rs            HidP_GetCaps (FFI)
    lease.rs               named mutex (FFI)
    tcp.rs                 loopback connection with timeouts
    system.rs              SPI_GETCLIENTAREAANIMATION (FFI)
    watch.rs               HID arrival/removal notification, CM_Register_Notification (FFI)

  profiles/                device data, not code (section 2)
    mod.rs, rule.rs        parsing built-in TOML once; lookup; passive match rules
    data/*.toml            device profiles and HID family rules

  drivers/                 one folder per protocol; registry() in mod.rs is the only list
    sonix_aa55/            mod.rs, codec.rs, info.rs, backup.rs, driver.rs, tests.rs
    sonix_feature/         mod.rs, codec.rs, driver.rs, tests.rs
    hid_lamparray/         mod.rs, descriptor.rs, codec.rs, driver.rs, tests.rs
    by_tech/               mod.rs, codec.rs, state.rs, backup.rs, driver.rs, tests.rs (experimental)
    fake_bus.rs            scripted Bus for driver tests (test builds only)

  classify/                passive family classification over profiles/data/families.toml
  diagnostics/             report.rs (JSON report), probe.rs (passive probe through the drivers)
  firmware/                static EXE/BIN analysis for developers (`--inspect` only, not in the
                           interface): pe.rs, image.rs, package.rs, error.rs
  storage/                 atomic.rs (atomic writes), backups.rs (backup directory)

  app/                     application state without UI
    identity.rs            product name, client name, lock prefix, data directory (section 7)
    session.rs             found devices, their status, operations on worker threads
    discovery.rs           one search: passive comparison first, then the drivers (read-only)
    watch.rs               device arrival/removal gathered into one automatic search
    diagnostics.rs         probe and report state
    events.rs              event channel from short-lived worker threads to the UI
    settings.rs            user settings (experimental drivers, theme)

  ui/                      display only (docs/DESIGN.md)
    app.rs, shell.rs       eframe::App; navigation pane
    shell/content.rs       page composition and single shared device surface
    device_reveal.rs       reversible surface/stage geometry and source tile registration
    strings.rs             every user-facing string (Russian default, translatable)
    theme/                 tokens.rs (the only literals; dark/light per token), mode.rs (theme blend),
                           text.rs, fonts.rs, style.rs
    icons/                 enum Icon, embedded Hugeicons outlines, path flattening
    motion/                spring.rs, tokens.rs, transition.rs, scroll.rs, store in mod.rs
    widgets/               button, card, badge, switch, expander, spinner, lighting_palette,
                           device_tile (the square device tile, docs/DESIGN.md 8.1)
    viewer/                glb.rs + gltf.rs (glTF 2.0 binary reader), catalog.rs, led.rs, preview.rs
      keycaps.rs           one-time independent plastic/outline-legend atlas for bound key surfaces
      gpu/                 safe wgpu host, mesh/mips, matrices, pipelines, targets and WGSL shaders
      raster.rs, raster/   software reference renderer for contexts without a GPU backend
    pages/                 devices, device, lighting (explicit hardware actions), diagnostics, settings
      studio/              viewport workspace state, model stage, mode island and settings inspector

  cli/                     --probe, --inspect, --lighting-probe, --review (+ raster.rs, review_state.rs)
assets/models/*.glb        device models named by profiles (`visual = "..."`), embedded at build time
assets/models/stages.toml  visual node/input bindings and camera focus; no hardware admission data
tests/admission.rs         every probe fixture is rejected by every foreign driver
tests/fixtures/            probe reports, reference descriptors, protocol packets
```

### File-splitting rules

1. **Codec separately.** `codec.rs` holds pure functions over bytes: building and parsing packets, with
   no HID, no clock, no files. It is tested with fixtures, without hardware.
2. **Exchange separately.** `driver.rs` implements the traits and glues the codec to the bus: request
   sequence, pauses, ACK checks. Its order is tested against `drivers/fake_bus.rs`.
3. **State separately from display.** Everything testable without a window lives in `app/` and
   `drivers/`. A file in `ui/` contains no protocol logic.
4. **One type, one file** when the type has its own behavior. Small related types may stay together.
5. **A file over ~300 lines** is split by responsibility, not alphabetically and not into `utils.rs`:
   files named `utils`, `helpers`, `common`, `misc` are forbidden.
6. **Public only what is needed outside.** Everything else is `pub(crate)` or private.

One crate plus per-driver feature flags (`driver-sonix-aa55`, `driver-sonix-feature`,
`driver-hid-lamparray`, `driver-by-tech`) is simpler than an early workspace split.
Extracting workspace crates makes sense once external drivers with their own release cycle appear.

## 2. Device-specific support: profiles, protocol adapters, a neutral core

| Layer | What it holds | Who adds it | Example |
| --- | --- | --- | --- |
| **Device profile** (data) | Vendor, model, kind, VID/PID or product filter, which driver | Any contributor, no Rust needed | `profiles/data/sonix.toml` |
| **Driver** (code) | Protocol: packets, exchange order, write safety | A developer, after review | `sonix_aa55`, `hid_lamparray` |
| **Core and UI** (code) | Capabilities, session, display | The project team | the "Lighting" section |

Protocols are often shared between brands, so drivers are named after the protocol. A device of a known
protocol is added as an entry in `profiles/data/*.toml` without code changes. A profile is a *hint*, not a
permission to write: the driver still checks report sizes and a safe reply, and hard safety exclusions
(for example the SONiX ROM bootloader PID) live in driver code, never in data.

Profiles are built into the EXE with `include_str!` and parsed once on first use. External profile files
are never read. A test checks that every built-in file parses (`deny_unknown_fields`).

Neutrality rules:
1. `core/`, `app/`, `ui/` contain no brand names, model names or USB identifiers. The exception is text
   that comes from a profile or a driver.
2. Anything needed by only one brand (the vendor software's mutex, firmware quirks such as the 80A0
   palette read-back) stays inside its driver.
3. User-interface strings describe a capability ("Lighting"), not a brand.
4. A new device kind means a new `Capability` plus a page section. Existing drivers do not change.

### No dynamic plugins (DLL)

Loading third-party libraries into a process that writes to USB devices is a risk to hardware and to the
user. Extension goes through source code: a driver passes review and tests and is built in.

## 3. Driver contract

```rust
pub trait Driver: Send + Sync {
    fn info(&self) -> DriverInfo;                       // id, protocol name, maturity, requirement
    fn admits(&self, endpoint: &Endpoint) -> bool;      // passive: identifiers and report sizes
    fn discover(&self, bus: &Arc<dyn Bus>) -> Discovery; // read-only; devices + typed notes
    fn identify(&self, bus: &dyn Bus, endpoint: &Endpoint)
        -> Option<Result<Identification, DriverError>>; // read-only, for the diagnostic report
}

pub trait Device: Send {
    fn descriptor(&self) -> &DeviceDescriptor;
    fn lighting(&mut self) -> Option<&mut dyn Lighting> { None }
}

pub trait Lighting {
    fn traits(&self) -> LightingTraits;                 // writable, persistent, rollback
    fn editor(&self) -> Option<LightingEditor>;         // cached programs and saved settings; no IO
    fn apply_settings(&mut self, settings: LightingSettings) -> Result<Outcome, DriverError>;
    fn apply_solid(&mut self, color: Rgb) -> Result<Outcome, DriverError>;
    fn restore(&mut self) -> Result<Outcome, DriverError>;
    fn restore_snapshot(&mut self, snapshot: &Snapshot) -> Result<Outcome, DriverError>;
}
```

A device holds an `Arc<dyn Bus>` and opens its HID channel **per operation**; the channel closes before
the lock is released. Drivers never touch the OS directly: every exchange goes through `Bus`, so tests
replay it on a fake bus. A device lacking a capability does not return it. The UI walks capabilities
and shows actions from `LightingTraits`: "Restore" when `rollback != Unavailable`, "From backup" for
`SessionAndFile`, and badges for persistence and missing rollback. No `match` on a driver or brand.

`core/lighting.rs` defines neutral native-program metadata. Drivers without native programs use the
default unavailable implementation. BY Tech builds its catalog from validated cached configuration;
`app/session` captures it at discovery and after operations. `ui/pages/studio/programs.rs` holds local
per-preset drafts and the scrolling island. Only the explicit hardware action invokes a worker.
`Driver::lighting_preview` exposes static control metadata without discovery or a bus. The application
catalog caches it once. The studio uses it for offline pages and live connections lacking lighting,
labels the values illustrative and keeps Apply disabled. Native device state always takes priority.

## 4. Migration status

Done in 0.3.0 (each item replaced the old code; codec tests carried over):
1. `app/identity.rs` holds the name, client name, lock prefix and data directory.
2. `ui/theme/tokens.rs` and `ui/motion/` replace the literals of the old `main.rs`.
3. `main.rs` split into `cli/`, `ui/shell.rs`, `ui/pages/*`.
4. All `unsafe` and FFI moved to `transport/`.
5. `core/` (`Driver`, `Device`, `Lighting`, `DriverError`, `Bus`) and `drivers::registry()`.
6. `sonix_aa55`, `sonix_feature`, `hid_lamparray` moved onto the traits; `enum Session` and
   the numeric adapter choice are gone; the vendor-software mutex and name filters live in drivers and
   profiles.
7. `profiles/` with device and family data; `families.rs` texts moved there (English and Russian).

Behavior changes worth knowing:
- HID handles are opened per operation instead of for the whole session (AA55, Feature, LampArray).
  The packet sequence is unchanged. **Not yet verified on the co-author's keyboard.**
- Search is automatic (item 11): at startup and whenever a relevant HID interface comes or goes; the
  "Refresh" button forces it. Experimental drivers (`sonix_feature`, `by_tech`) take part only when
  enabled in settings; until then their devices are listed as recognized and nothing is sent to them.
- LampArray accepts every `LampArrayKind`.
- The probe sends AA55 `GET_INFO` to endpoints the AA55 driver admits (profile match) instead of a
  fixed PID list.
- Diagnostic report schema 2: `identification` replaces `info`/`info_payload_hex`; `target_marking` is
  gone. Schema 1 reports still load.

Done after 0.3.0 (2026-10-28):
8. `drivers/by_tech/` for `258A:010C` (wired AK820 MAX `GPWB2401`): protocol restored by static
   analysis of the vendor package (`docs/DEVICES.md`, section 1.1). Experimental until the owner
   confirms a write; family check by the vendor's own password request; read-modify-write of the
   lighting blocks with a backup and a read-back.
9. Device tiles with 3D models (`ui/viewer`, `ui/widgets/device_tile.rs`, `docs/DESIGN.md` 8.1).
   `DeviceDescriptor::visual` carries the model id from the profile.
10. The firmware/package analysis block left the interface (the application is for users;
    `docs/DESIGN.md`, section 8). `firmware/` stays for the developer command `--inspect`.

11. Automatic search without polling: `transport/watch.rs` registers `CM_Register_Notification`
    for `GUID_DEVINTERFACE_HID` (Windows calls back on its own thread pool), `app/watch.rs` gathers
    the burst of notifications of one device (400 ms of quiet), and `app/discovery.rs` first compares
    the passive enumeration with the previous search: only if an interface some driver admits came
    or went does the full read-only search run. Plugging in an unrelated device sends nothing and
    keeps every device (and session-only lighting) as it is. An open device page follows its device
    to its new slot or returns to the list when the device is gone.

Next (targets):
12. Optional logging to the data directory with rotation (section 6).

## 5. Performance

**Budgets** and measurements. The historical OpenGL/CPU revision used 69 MB with an empty page and
94 MB with a keyboard tile (three runs each, Windows 11, 2026-10-28). These are not measurements of
the current backend. The owner explicitly requested GPU acceleration for the workspace revision.
Current native wgpu/Direct3D 12 measurements and their limits are in `GPU-PERFORMANCE.md`.

| State | CPU | Working-set memory |
| --- | --- | --- |
| Window minimized or hidden | 0% (no repaint, no polling) | as low as possible |
| Window open, no actions | measured 0 ms CPU over 5 s in a synthetic native GPU workspace (budget < 0.5% of a core) | 182.83 MB working set, 257.21 MB private memory on RTX 3080; exceeds the earlier 120 MB guideline; one sample, including driver/runtime costs |
| Applying a color | short spike | no growth after completion (not measured: needs the device) |
| Device model, idle | no new 3D submissions or readbacks; final GPU texture is reused | static mesh, mipmaps and final image retained; large work attachments released |
| Waiting for devices | 0: no thread, timer or polling; Windows calls back on a device change | nothing extra |
| Device model while hovered or turning | full resolution with 4x MSAA; native UI update mean 0.29 ms, max 2.01 ms in the synthetic preview (excludes GPU execution/presentation) | targets grow in 128 px buckets; scratch attachments exist only while rendering |

**Rules:**

1. **No endless loops or timers.** egui runs reactively. While a spring moves, `ui/motion` requests the
   next frame; when every spring rests nothing is drawn until an event. Widgets never call
   `request_repaint()` themselves. The busy indicator runs only while an operation runs. A minimized
   window computes no motion; values jump to their targets on return.
2. **No background device polling.** Search runs at startup, on an operating-system device-change
   notification (no thread or timer of ours) and on the "Refresh" button. An automatic search sends
   nothing to devices when no relevant interface changed.
3. **Threads are short-lived**: `app::Events::spawn` starts one thread per operation.
4. **A HID handle is opened per operation and closed right after.**
5. **No needless copies.** Packets are fixed-size arrays; descriptors are parsed once per discovery.
6. **Dependencies are minimal.** New in 0.3.0: `toml` (profiles, parse only) and `window-vibrancy`
   (Mica through DWM). Drivers use feature flags. `release` profile: `lto`, `strip`.
7. **Logs and reports are size-bounded**; backups are small JSON files (up to 128 KB, enforced on load).
8. **Images and fonts.** Icons are embedded vector outlines of only the glyphs in use. Fonts come from
   the system (Segoe UI, Segoe UI Semibold, Cascadia Mono); none are bundled.
9. **Mica is done by DWM, not by the application.** Custom backdrop blur shaders are forbidden.
   The model's bounded optical bloom is separate from the interface backdrop.
10. **Device models** are embedded `.glb` files (`assets/models/`, 2.9 MB for the AK820 MAX) parsed on a
    short-lived thread the first time a tile shows them; the loader's `request_repaint` is the same
    kind of wake-up as a finished worker. Safe Rust in `ui/viewer/gpu` uploads each visual once and
    renders changed poses/lighting only, using WGSL shaders. Native output is registered with egui's
    renderer directly; it never reads frames back to the CPU. No `unsafe` is added. Scratch targets
    are released at rest/minimize; only cached mesh/material data and resolved images remain. The
    frame that changes a pose goes through `motion::keep_alive`. A parsed model remains cached.
11. Any change that may affect consumption is measured as in the table above and recorded here.

## 6. Errors, logs, tests

- Driver errors are typed (`DriverError`); user-facing text is built in `ui/strings.rs`. Low-level
  details (English) go to diagnostic JSON, not to the interface.
- Logging is a target: off by default, written to `%LOCALAPPDATA%\LusoriumStudio\logs` with rotation,
  without serial numbers or device paths.
- Codec tests use fixtures from `tests/fixtures/`. Exchange tests run on `drivers/fake_bus.rs`.
  `tests/admission.rs` fails if a new `probe-*.json` fixture is not listed with its owning driver.
  A test that needs hardware is marked `#[ignore]` and run manually by the device owner.
- `--review DIR` renders interface states with synthetic data to PNG for design review.

## 7. Name and brand: where they live

The product is "Lusorium Studio" (until 2026-10-28 the working name "AJAZZ Connect"). A rename touches
these places only:

| What | Where |
| --- | --- |
| Product name, service client identity, lock prefix, data directory `LusoriumStudio`, report file stem | `src/app/identity.rs`; the previous data directory goes to `LEGACY_DATA_DIRS` and is moved on first launch (`AjazzConnect` today) |
| Package and exe name `lusorium-studio` | `Cargo.toml` |
| App icon (Hugeicons `game-controller-03` on a dark tile): window, taskbar and EXE resource | `src/ui/icons/app_icon.rs`, embedded by `build.rs` |
| Archive names in `package*.ps1` | the scripts (previous developer's names and paths; not used for regular builds) |
| Vendor-software mutex `Local\AjazzStudio-AK820MAX` | `src/drivers/sonix_aa55/mod.rs`, `src/drivers/sonix_feature/mod.rs` |
| Product-name filters `AK820`, `AJAZZ` | `src/profiles/data/*.toml` |
| Classifier texts about specific models | `src/profiles/data/families.toml` |


### Device reveal review (2026-10-28)

Historical CPU-renderer check: `--review verification/device-reveal-checked` exercises tile pointer input, Back, and an Escape
reversal after seven incoming frames at 720, 1040 and 1600 px, using synthetic devices only.
Captured intermediate frames contain one selected device model; the source tile suppresses its own
stage while the shared stage travels. Hardware-free frame timing (egui update and CPU model rendering,
excluding PNG rasterization, presentation and OS compositing) averaged 11.1-15.5 ms, with a maximum of
22.0 ms across these six traces. This does not establish live window frame rate. At frame 140 all six
traces requested no further repaint (`Duration::MAX`); live Task Manager CPU/memory were not remeasured.

Offline placeholders live in `profiles/data/previews.toml` as `[[preview]]` entries. Their schema has
only display metadata (vendor, model, kind, visual), with no admission rules or driver. The catalog
includes these with an empty driver-variant list. `ui/pages/device_view.rs` borrows display fields
from either a live descriptor or a catalog entry, representing the missing driver explicitly.
Placeholders cannot enter discovery or expose hardware actions. Visual editing is allowed by asset bindings.

Device workspace revision (2026-10-28): model editing is local view state in `ui/pages/studio/`.
Live and catalog pages share the workspace. Asset-only lighting/encoder previews may appear even
without a device driver; hardware actions require the live entry's lighting capability. Input received
by the focused window is used transiently for per-node key glow/depression and never enters reports,
storage or transport. The global input hook and HID key capture are intentionally absent.

`viewer/preview.rs` interprets visual bindings; the GLB reader retains node identity per vertex.
Texture cache keys include quantized camera focus/zoom and per-part lighting/depression. The previous
CPU renderer used up to eight scoped row workers with 2x2 supersampling; it remains a reference path
for contexts without an installed GPU backend. Native windows always install the wgpu backend.

Historical CPU workspace review on 2026-10-28 (`--review verification/device-workspace`): large-scene motion
still exceeds a 16.7 ms frame budget. The banded renderer approximately halves the original wide-view
cost while retaining full resolution, but this is not a 60 fps implementation. Per-scene means/maxima
are saved beside the screenshots. Timings measure egui update and CPU rasterization, excluding PNG
export, presentation and OS compositing; native-window CPU/memory and live frame rate were not measured.
Every workspace scene must request no further repaint after settling; this is a scheduling check,
not a live Task Manager measurement. Encoder zoom, lighting, settings, reversal, focused-key highlights
and the 520 px minimum height are included. The current GPU suite contains 31 scenes at three widths
(`verification/gpu-checked/complete.json`), including pointer tracking after lighting/encoder zoom.

Lighting fidelity revision (2026-10-28): `viewer/led.rs` prepares legend apertures from the built-in
atlas and four local emitter weights per vertex once during asset loading. Emitters are below the
keycaps, so upward-facing plastic does not receive a uniform diffuse tint. Both renderers use
perspective-correct texture interpolation; the 1536 px keyboard atlas is retained instead of being
halved to 768 px. Visible socket and legend emission feeds a bounded, quarter-resolution separable
bloom pass. This is a local illumination approximation, without ray-traced shadows or measured cap
transmission. The native GPU path adds GGX material shading, a filtered shadow map, screen-space
contact shading, 4x MSAA and anisotropic mip sampling. Emission, bloom, camera changes and key
depression all remain texture-cached at rest. GPU structure and measurements: `GPU-PERFORMANCE.md`.

`widgets/lighting_palette.rs` owns hue-only LED colors. Its private palette guarantees full value and
independent ring editing. The old HSV plane and hex-entry widget were removed because they allowed
black/dim colors and no longer had consumers. Solid/Reactive expose one color and Wave three. Driver
write actions receive only the first Solid color, after the same explicit button and backup checks.
`--review DIR [SCENE_PREFIX]` optionally restricts the offline suite for focused visual checks.

AJ159 validation (2026-10-28): `--review verification/aj159` completed with exit code 0.
Catalog and placeholder passport images were visually checked at 720, 1040 and 1600 px.
The integrated GLB is 15,670,536 bytes; the existing on-demand loader and pose texture cache are reused.
Live hover frame time and working-set growth with this larger model have not been measured.

### Continuous device return and model bounce (2026-10-28)

The source list blocks input without egui's disabled alpha and restores its explicit dimming along
the surface spring. Its tile chrome stays present; only the selected stage transfers to the detail
page. A smoothstep fade over the last 22% of surface travel exposes labels before the handoff.
The Refresh button retains its visual state while passive, avoiding a final disabled-to-enabled pop.
A separate model spring (`T=0.52 s`, damping `0.85`) provides about 0.6% overshoot on opening;
return retargets both springs without resetting velocity. The background does not overshoot.
`--review verification/device-return-bounce-final` adds late-return frames at all three widths. In
the hardware-free traces, mean update/render time was 10.7-17.7 ms, maximum 25.2 ms; all six traces settled with no further repaint requested. These timings exclude
presentation and do not establish live FPS. Unit tests check handoff opacity limits, model overshoot,
velocity preservation and reduced motion. No device protocol or hardware operations changed.

### Battery discovery snapshots (2026-10-28)

`core::Detail::Battery` carries a reported percentage and `Detail::Power` a generic power source;
neither establishes active charging. The BY Tech wired and radio implementations share a pure
battery payload parser, with separate wire codecs and read paths in `drivers/by_tech/`. Radio is a
read-only `Device` with no `Lighting` implementation. Both remain behind the existing experimental
driver setting. Snapshots refresh only through discovery; no timer, idle thread or polling is added.
The passive probe records radio descriptors only, while `--lighting-probe by_tech` exercises the
radio discovery read; this avoids reading the same descriptor twice in one diagnostic enumeration.

### Live battery scheduling (owner-requested exception, 2026-10-28)

The owner explicitly requested current battery data with adaptive polling, superseding the older
battery-only snapshot restriction. `core::Battery` is an optional capability. `app/session/battery.rs`
transfers available devices into short-lived read jobs through the existing Events channel, excluding
concurrent discovery and lighting operations. `ui/app/battery.rs` schedules only deadline repaints:
5 seconds visible, 300 seconds minimized, immediate restore, 60-second failure backoff visible.
Devices without the capability create no telemetry wakeups. `--review` never starts telemetry.
Radio descriptor validation happens only when creating the session; refresh uses no descriptor,
settings read or global USB enumeration. HID channels close at the end of each operation.

### Per-device telemetry and user presets (2026-10-28)

The earlier global battery deadline is replaced by individual clocks in `app/battery.rs` and a
bounded dispatcher in `app/session/battery.rs`. At most one telemetry operation is in flight, also
serializing shared receivers. Completion is delivered per device; an unresponsive unit no longer
holds an entire batch's results or changes other devices' retry deadlines. Errors back off per device
(up to 15 minutes, never faster than its selected interval). Restore/mode changes stagger requests.
Last successful values and monotonic timestamps are retained separately from the current display;
failed reads still do not present stale percentages as fresh. The future tray can access those values.

Settings persist one global mode (legacy settings default to Automatic):

| Mode | Open device card | Other pages/devices | Background on battery | Background while charging |
| --- | --- | --- | --- | --- |
| Automatic | 5 s | 60 s | 5 min | 1 min |
| Responsive | 2 s | 30 s | 2 min | 30 s |
| Eco | 15 s | 120 s | 15 min | 5 min |

Charging intervals use external power and a reported percentage below 100. There is no notification
requirement. Restoring the window requests an immediate staggered refresh. A hidden/minimized frame
processes events and deadlines but skips construction of the shell, pages and device models. The
public `App::set_window_hidden` hook supports a future tray hide operation independently of the native
minimized flag; native tray icons themselves are not implemented by this change. Worker completion
still wakes the event loop to return device ownership and dispatch the next due read; no permanent
worker, per-device timer thread, global USB rescan, or repeated descriptor read is introduced.
