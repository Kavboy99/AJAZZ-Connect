# Architecture

This document describes how the code is organized (revision 0.3.0) and what is still a **target**.
Anything marked "target" is not implemented yet. Do not present the target as existing.

**Direction.** The application is a universal tool for configuring and customizing PC devices from any
manufacturer and of any type (keyboards, mice, lighting, fans, ...), not a single-brand program.
Everything tied to a brand is isolated in drivers and device profiles; the rest of the code is neutral
(section 2).

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
    openrgb/               mod.rs, wire.rs, client.rs, backup.rs, driver.rs, tests.rs
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
    app.rs, shell.rs       eframe::App; navigation pane, content layer, page transitions
    strings.rs             every user-facing string (Russian default, translatable)
    theme/                 tokens.rs (the only literals; dark/light per token), mode.rs (theme blend),
                           text.rs, fonts.rs, style.rs
    icons/                 enum Icon, embedded Hugeicons outlines, path flattening
    motion/                spring.rs, tokens.rs, transition.rs, scroll.rs, store in mod.rs
    widgets/               button, card, badge, switch, expander, text_field, spinner, color_picker/,
                           device_tile (the square device tile, docs/DESIGN.md 8.1)
    viewer/                3D device models: glb.rs + gltf.rs (glTF 2.0 binary reader), raster.rs
                           (software rasterizer), light.rs, math.rs, catalog.rs (built-in models)
    pages/                 devices, device, lighting (a capability section), diagnostics, settings

  cli/                     --probe, --inspect, --lighting-probe, --review (+ raster.rs, review_state.rs)
assets/models/*.glb        device models named by profiles (`visual = "..."`), embedded at build time
tests/admission.rs         every probe fixture is rejected by every foreign driver
tests/fixtures/            probe reports, reference descriptors, SDK packets
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
`driver-hid-lamparray`, `driver-openrgb`, all on by default) is simpler than an early workspace split.
Extracting workspace crates makes sense once external drivers with their own release cycle appear.

## 2. Universality: profiles, drivers, a neutral core

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

## 4. Migration status

Done in 0.3.0 (each item replaced the old code; codec tests carried over):
1. `app/identity.rs` holds the name, client name, lock prefix and data directory.
2. `ui/theme/tokens.rs` and `ui/motion/` replace the literals of the old `main.rs`.
3. `main.rs` split into `cli/`, `ui/shell.rs`, `ui/pages/*`.
4. All `unsafe` and FFI moved to `transport/`.
5. `core/` (`Driver`, `Device`, `Lighting`, `DriverError`, `Bus`) and `drivers::registry()`.
6. `sonix_aa55`, `sonix_feature`, `hid_lamparray`, `openrgb` moved onto the traits; `enum Session` and
   the numeric adapter choice are gone; the vendor-software mutex and name filters live in drivers and
   profiles.
7. `profiles/` with device and family data; `families.rs` texts moved there (English and Russian).

Behavior changes worth knowing:
- HID handles are opened per operation instead of for the whole session (AA55, Feature, LampArray).
  The packet sequence is unchanged. **Not yet verified on the co-author's keyboard.**
- Search is automatic (item 11): at startup and whenever a relevant HID interface comes or goes; the
  "Refresh" button forces it. Experimental drivers (`sonix_feature`, `by_tech`) take part only when
  enabled in settings; until then their devices are listed as recognized and nothing is sent to them.
- LampArray accepts every `LampArrayKind`, OpenRGB every device type (previously keyboards only).
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

**Budgets** and the values measured on 2026-10-28 (Windows 11, release build, window open on the
"Devices" page, three runs each, compared with 0.2.0 on the same machine):

| State | CPU | Working-set memory |
| --- | --- | --- |
| Window minimized or hidden | 0% (no repaint, no polling) | as low as possible |
| Window open, no actions | measured 0 ms CPU over 5 s (budget < 0.5% of a core) | measured 69 MB (0.2.0: 82 MB; guideline < 80 MB). With the automatic search at startup and one device tile, measured 2026-10-28: 86 MB without a model, 94 MB with the keyboard model (three runs each, ± 1 MB); about 3 s of CPU over the first seconds of startup in both builds, 0 afterwards |
| Applying a color | short spike | no growth after completion (not measured: needs the device) |
| Device tile with a model, idle | 0: the texture is reused, nothing is rendered | the parsed model ≈ 8 MB (once per model: 35k vertices, 64k triangles, the texture capped at 1024 px) plus ≈ 0.4 MB per tile texture |
| Waiting for devices | 0: no thread, timer or polling; Windows calls back on a device change | nothing extra |
| Device tile while hovered or turning | measured 4.1 ms per frame for a 320 px tile (half-resolution draft), 18 ms once when it stops (2×2 supersampled), release build, one core | no growth |

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
9. **Mica is done by DWM, not by the application.** Custom blur shaders are forbidden.
10. **Device models** are embedded `.glb` files (`assets/models/`, 2.9 MB for the AK820 MAX) parsed on a
    short-lived thread the first time a tile shows them; the loader's `request_repaint` is the same
    kind of wake-up as a finished worker. They are drawn by the CPU rasterizer in `ui/viewer` (no GPU
    code, so no `unsafe` outside `transport/`) only while the pose changes; the extra frame that
    upgrades a draft goes through `motion::keep_alive`. A model stays in memory once parsed.
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
| Product name, OpenRGB client name, lock prefix, data directory `LusoriumStudio`, report file stem | `src/app/identity.rs`; the previous data directory goes to `LEGACY_DATA_DIRS` and is moved on first launch (`AjazzConnect` today) |
| Package and exe name `lusorium-studio` | `Cargo.toml` |
| App icon (Hugeicons `game-controller-03` on a dark tile): window, taskbar and EXE resource | `src/ui/icons/app_icon.rs`, embedded by `build.rs` |
| Archive names in `package*.ps1` | the scripts (previous developer's names and paths; not used for regular builds) |
| Vendor-software mutex `Local\AjazzStudio-AK820MAX` | `src/drivers/sonix_aa55/mod.rs`, `src/drivers/sonix_feature/mod.rs` |
| Product-name filters `AK820`, `AJAZZ` | `src/profiles/data/*.toml` |
| Classifier texts about specific models | `src/profiles/data/families.toml` |
