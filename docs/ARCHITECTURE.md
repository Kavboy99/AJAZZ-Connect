# Architecture

This document describes two states: **how the code is organized now** (revision 0.2.0) and the **target
structure** the code is being moved to in small steps. Anything marked "target" is not implemented yet.
Do not present the target as existing.

**Direction.** The application is a universal tool for configuring and customizing PC devices from any
manufacturer and of any type (keyboards, mice, lighting, fans, ...), not a single-brand program. The
current code is a first slice for AJAZZ keyboards. Everything tied to a brand is isolated in drivers and
device profiles; the rest of the code is neutral (section 2).

## 1. How it is organized now

```
src/
  main.rs               611 lines: egui window, CLI (--probe/--inspect/--review/--lighting-probe), review rendering
  lib.rs                module list
  lighting.rs           enum Session { OpenRgb, LampArray, Aa55, Feature } + a match in every method
  aa55.rs               SONiX AA55 adapter (codec + exchange + backup)
  sonix_feature.rs      SONiX Feature adapter (experimental)
  lamparray.rs          HID LampArray adapter
  openrgb.rs            OpenRGB SDK client (TCP 127.0.0.1:6742)
  protocol.rs           GET_INFO parsing (AA55)
  families.rs           passive interface classification
  firmware_assistant.rs USB diagnostics, EXE/BIN firmware parsing, HidP FFI, Report
  hardware.rs           DeviceLease mutex (compatibility with AJAZZ Studio)
  storage.rs            atomic JSON writes
tests/protocols.rs      codec tests on fixtures
tools/*.py              standalone analysis labs (never open HID)
```

Weak spots to fix, not to copy:

| Problem | Where | Consequence |
| --- | --- | --- |
| Drivers are listed with `match` in one `enum` | `lighting.rs` | A new driver means edits in 7 places and in the UI |
| The adapter is selected by a number `0..3` | `lighting.rs`, `main.rs` | Unclear contract, cannot be extended from outside |
| Window, CLI and review rendering in one file | `main.rs` | Cannot be read or tested in parts |
| Codec, HID exchange and scenario in one file | `aa55.rs`, `lamparray.rs` | The codec cannot be tested without IO |
| `unsafe` FFI in several places | `hardware.rs`, `firmware_assistant.rs` | The unsafe boundary is hard to audit |
| UI colors and sizes written as literals | `main.rs` | No single design system (`docs/DESIGN.md`) |
| Previous developer's paths in scripts | `build.ps1`, `package.ps1` | Scripts are not portable |
| Product name and brand hard-coded | see section 7 | Renaming and extending to other brands requires edits across the code |
| Only keyboards and only lighting in the data model | `lighting.rs`, `openrgb.rs` ("keyboards only" filter) | Mice, fans and other device kinds do not fit |

## 2. Target structure

One crate, modules by responsibility. A codec does not know about HID, a driver does not know about the
UI, the UI does not know about specific drivers.

```
src/
  main.rs                  entry point only: parse arguments -> cli::run or ui::run
  lib.rs                   public modules

  core/                    shared types and contracts; no IO and no brand names
    device.rs              DeviceDescriptor, DeviceId, DeviceKind (keyboard, mouse, fan, ...)
    capability.rs          Capability and its parameters: Lighting, KeyRemap, Dpi, FanCurve, ...
    driver.rs              trait Driver, trait Device (section 3)
    color.rs               Rgb
    error.rs               DriverError (typed errors instead of strings)
    snapshot.rs            Snapshot: saved state for rollback

  transport/               the only place with unsafe and real IO
    hid.rs                 hidapi wrapper: enumeration, opening, reports
    hid_caps.rs            HidP_GetCaps, descriptors (FFI)
    tcp.rs                 local socket for OpenRGB
    lease.rs               DeviceLease (mutex)

  profiles/                device data, not code (section 2.1)
    mod.rs                 loading built-in profiles, lookup by VID/PID
    data/<vendor>.toml     device profiles: name, VID/PID, driver, capabilities

  drivers/                 one folder per protocol (not per brand); registry in mod.rs
    mod.rs                 registry(): the only list of drivers
    sonix_aa55/            mod.rs, codec.rs, driver.rs, backup.rs, tests.rs
    sonix_feature/         mod.rs, codec.rs, driver.rs, tests.rs
    hid_lamparray/         mod.rs, descriptor.rs, codec.rs, driver.rs, tests.rs
    openrgb/               mod.rs, wire.rs, client.rs, driver.rs, tests.rs
    by_tech/               target: driver for the repository owner's mechanical keyboard

  classify/                passive family classification (formerly families.rs)
  firmware/                static EXE/BIN analysis, no writes to a device
  storage/                 atomic.rs (JSON), backups.rs (%LOCALAPPDATA%\AjazzConnect)

  app/                     application state without UI: session, events, tasks
    identity.rs            product name, exe name, data directory, OpenRGB client name (section 7)
    session.rs             list of drivers -> found devices -> selection
    events.rs              event channel from worker threads to the UI

  ui/                      display only (docs/DESIGN.md)
    strings.rs             user-facing text table (translatable; Russian is the current default)
    theme/tokens.rs        colors, spacing, radii, fonts: the single source
    theme/style.rs         applying tokens to egui
    icons.rs               enum Icon -> Hugeicons glyph
    motion/                motion (docs/DESIGN.md, section 9)
      spring.rs            Spring { value, velocity, target }: stable step, stop condition
      tokens.rs            spring.fast / standard / page / release
      animated.rs          Animated<T>: a value that eases toward its target, interruptible
      transition.rs        transitions between pages and dialogs
      scroll.rs            kinetic scrolling with decay
    widgets/               card.rs, button.rs, color_picker.rs, status_badge.rs, ...
    pages/                 lighting.rs, diagnostics.rs, about.rs
    shell.rs               window, navigation, acrylic backdrop

  cli/                     probe.rs, inspect.rs, review.rs
tests/                     integration tests and fixtures/
```

### File-splitting rules

1. **Codec separately.** `codec.rs` holds pure functions over `&[u8]` and `Vec<u8>`: building and parsing
   packets, with no HID, no clock, no files. It is tested with fixtures, without hardware.
2. **Exchange separately.** `driver.rs` implements the `trait` and glues the codec to the transport:
   request sequence, pauses, ACK checks.
3. **State separately from display.** Everything testable without a window lives in `app/` and
   `drivers/`. A file in `ui/` contains no protocol logic.
4. **One type, one file** when the type has its own behavior. Small related types may stay together.
5. **A file over ~300 lines** is split by responsibility, not alphabetically and not into `utils.rs`:
   files named `utils`, `helpers`, `common`, `misc` are forbidden.
6. **Public only what is needed outside.** Everything else is `pub(crate)` or private.

One crate plus per-driver feature flags (`driver-sonix-aa55`, `driver-by-tech`, ...) is better than an
early workspace split: simpler build, less glue code, and the final EXE can be slimmed down if needed.
Extracting workspace crates (`ajazz-core`, `ajazz-drivers`) makes sense once external drivers with their
own release cycle appear.

### 2.1 Universality: profiles, drivers, a neutral core

Three layers with different roles:

| Layer | What it holds | Who adds it | Example |
| --- | --- | --- | --- |
| **Device profile** (data) | Name, manufacturer, VID/PID, kind, which driver and which capabilities | Any contributor, no Rust needed | "Vendor X, model Y, `1234:5678`, driver `hid_lamparray`" |
| **Driver** (code) | Protocol: packets, exchange order, write safety | A developer, after review | `sonix_aa55`, `hid_lamparray` |
| **Core and UI** (code) | Capabilities, session, display | The project team | the "Lighting" page |

Protocols are often shared between brands (one chip in keyboards and mice of different brands; the HID
LampArray standard belongs to nobody), so drivers are named after the protocol, not the brand. A device
of a known protocol is added as an entry in `profiles/data/*.toml` without code changes. A profile is a
*hint for choosing a driver*, not a permission to write: admission is still checked by the driver
(`AGENTS.md`, safety rules).

Profiles are built into the EXE at compile time and parsed lazily (memory). The application does not
execute external profile files from disk and does not trust them without a schema check.

Neutrality rules:
1. `core/`, `app/`, `ui/` contain no brand names, model names or USB identifiers. The exception is text
   that comes from a profile or a driver.
2. Anything needed by only one brand (the vendor software's mutex, lookup tables, firmware-specific
   fixes) stays inside its driver.
3. User-interface strings describe a capability ("Lighting", "DPI"), not a brand.
4. A new device kind means a new `Capability` plus a page. Existing drivers do not change.

### No dynamic plugins (DLL)

Loading third-party libraries into a process that writes to USB devices is a risk to hardware and to the
user. Extension goes through source code: a driver passes review and tests and is built in. Out-of-process
plugins are considered only as a separate decision.

## 3. Driver contract (target)

This is a sketch, not finished code. Exact signatures are refined during the first refactoring; the
principle is fixed: a driver describes itself, finds devices passively and is responsible for the safety
of its own writes.

```rust
/// A device family: only finds and creates devices.
pub trait Driver: Send + Sync {
    fn info(&self) -> DriverInfo; // id, name, maturity, authors, documentation link
    /// Read-only. No write commands during discovery.
    fn discover(&self, bus: &dyn Bus) -> Result<Vec<Box<dyn Device>>, DriverError>;
}

/// A found and identified device: a passport plus a set of capabilities.
pub trait Device {
    fn descriptor(&self) -> &DeviceDescriptor;     // kind, manufacturer, name, connection
    fn lighting(&mut self) -> Option<&mut dyn Lighting> { None }
    fn key_remap(&mut self) -> Option<&mut dyn KeyRemap> { None }
    fn dpi(&mut self) -> Option<&mut dyn Dpi> { None }
    // ... one method per capability
}

/// A capability is a separate small trait (interface segregation).
pub trait Lighting {
    fn traits(&self) -> LightingTraits;            // solid color, zones, effects, whether it persists
    fn apply_solid(&mut self, rgb: Rgb) -> Result<Applied, DriverError>;
    fn snapshot(&mut self) -> Result<Option<Snapshot>, DriverError>;
    fn restore(&mut self, snapshot: &Snapshot) -> Result<(), DriverError>;
}
```

A device that lacks a capability simply does not return it (`None`): no "not supported" stubs. A new
device kind adds a new capability in `core/capability.rs` and a page in `ui/pages/` without touching
existing drivers.

Consequences for the UI: it walks the device's capabilities and draws only the available pages and
actions. The "Restore" button appears when `Lighting::snapshot` returns `Some`. No `match` on the driver
type, the manufacturer or `DeviceKind`.

The registry (`drivers/mod.rs`) is the only place where drivers are listed:

```rust
pub fn registry() -> Vec<Box<dyn Driver>> {
    vec![
        #[cfg(feature = "driver-sonix-aa55")] Box::new(sonix_aa55::Aa55Driver::new()),
        // ...
    ]
}
```

## 4. Migration order (each step is a separate commit, tests stay green)

1. Introduce `app/identity.rs` and move the name, data directory and OpenRGB client name onto it;
   migrate the old directory (section 7).
2. Extract `ui/theme/tokens.rs` and `ui/motion/` from the literals in `main.rs` (no logic impact).
3. Split `main.rs`: `cli/`, `ui/shell.rs`, `ui/pages/*`.
4. Move `unsafe` and FFI into `transport/`.
5. Introduce `core/` (`Driver`, `Device`, `Capability`, `DriverError`) and `drivers/registry()`.
6. Move `aa55` into `drivers/sonix_aa55/` through the traits; remove its branches from `Session`; move
   brand-specific details (the vendor software's mutex, name filters) inside the driver.
7. Do the same for `sonix_feature`, `lamparray`, `openrgb`. Then delete `enum Session`.
8. Introduce `profiles/` (device data) and move the texts of `families.rs` into it.
9. Add `drivers/by_tech/` (read-only first).

Rule for each step: behavior for the co-author's keyboard does not change, and the tests in
`tests/protocols.rs` stay green.

## 5. Performance

The application runs next to games. The goal is to be unnoticeable.

**Budgets (targets; measure and record the actual values in this section):**

| State | CPU | Working-set memory |
| --- | --- | --- |
| Window minimized or hidden | 0% (no repaint, no polling) | as low as possible, release textures |
| Window open, no actions | < 0.5% of one core | guideline < 80 MB |
| Applying a color | short spike | no growth after completion |

**Rules:**

1. **No endless loops or timers.** Repaint on events (`egui` in reactive mode). Smooth animation is a
   special case, not an exception: while at least one spring (`ui/motion`) has not reached its target,
   the next frame is requested; once all have stopped, requests cease and the window draws nothing again.
   Active animations are tracked in one place (`ui/motion`), not by calling `request_repaint()` from
   arbitrary widgets. A spinner runs only while a task runs. A minimized window does not compute
   animations.
2. **No background device polling.** Keyboard search happens on a button press; automatic detection uses
   the Windows `WM_DEVICECHANGE` event, not a `sleep` loop.
3. **Threads are short-lived**: an IO task starts for the duration of the operation and ends; no
   permanent thread pool.
4. **A HID handle is opened per operation and closed right after.** We do not hold the device in the
   background: a game or the vendor software may need it.
5. **No needless copies.** Packets are fixed-size arrays on the stack; `Vec` only where the size is
   unknown. Descriptors are parsed once and cached in the session.
6. **Dependencies are minimal.** A new crate needs a justification in the PR (size, number of transitive
   dependencies). Drivers use feature flags. `release` profile: `lto`, `strip`.
7. **Logs and reports are size-bounded**; backups are small JSON files (up to 128 KB).
8. **Images and fonts.** Icons are a font subset (only the glyphs in use), not an SVG set and not
   separate PNGs. UI fonts come from the system (Segoe UI); we bundle none of our own.
9. **Acrylic is done by DWM, not by the application.** Custom blur shaders are forbidden.
10. Any change that may affect consumption is checked in Task Manager (5 minutes idle, minimized and
    restored); the result goes into the change description.

## 6. Errors, logs, tests

- Driver errors are typed (`DriverError`); user-facing text is built in `ui/`. Low-level details (Windows
  codes, bytes) go to the diagnostic JSON, not to a dialog.
- Logging is off by default; it is enabled by a flag and writes to `%LOCALAPPDATA%\AjazzConnect\logs`
  with rotation. Logs contain no serial numbers or device paths.
- Codec tests use fixtures from `tests/fixtures/` with hashes of their source. A test that needs hardware
  is marked `#[ignore]` and run manually by the device owner.

## 7. Name and brand: where they are hard-coded today

The name "AJAZZ Connect" is a working name. So that renaming and extending to other brands do not require
searching the whole code, once `app/identity.rs` exists every place below moves onto it (or inside the
driver of the corresponding brand). Inventory as of 2026-10-28:

| What | Where now | Where it should go |
| --- | --- | --- |
| Data directory `%LOCALAPPDATA%\AjazzConnect` (backups) | `aa55.rs`, `openrgb.rs`, texts in `lighting.rs`, `main.rs` | `identity.rs` + migrating the old directory on first launch |
| OpenRGB client name `AJAZZ Connect` | `openrgb.rs` (SDK handshake) | `identity.rs` |
| Window title, splash, "AJAZZ" labels | `main.rs` | `identity.rs` and device profiles |
| Crate and exe name `ak820max-assistant`, `AJAZZ-Connect.exe`, archive names | `Cargo.toml`, `package*.ps1` | `Cargo.toml` (one place); scripts take the name from there |
| Mutex `Local\AjazzStudio-AK820MAX` (compatibility with the vendor software) | `hardware.rs`, 7 calls of `DeviceLease::acquire("AK820MAX")` | Inside the `sonix_*` drivers; other brands get their own locks |
| Name filters `AK820`/`AJAZZ` | `aa55.rs`, `sonix_feature.rs`, `firmware_assistant.rs` | Device profiles |
| Texts about specific models in the classifier | `families.rs` | Profile data |
| Report name `AK820MAX-GPWB2401.json` | `main.rs` | Generate from the device data |

Order: first `identity.rs` and the data migration, then move the brand-specific parts into drivers. The
new name itself is chosen as a separate decision and substituted in one place.
