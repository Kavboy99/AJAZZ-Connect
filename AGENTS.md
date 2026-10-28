# AGENTS.md — project "Lusorium Studio"

Single set of rules for developers and AI agents (Claude Code, Codex, etc.). `CLAUDE.md` only
imports this file. Details live in `docs/`; this file holds what you must know before the first edit.

## What this project is

A lightweight Windows application for **configuring and customizing any PC device**: keyboards,
mice, lighting, fans and other peripherals from different manufacturers (lighting, remapping, passive
USB/HID diagnostics). The manufacturer is not a property of the application but of a driver: everything
that knows about a specific brand lives only in `drivers/` and in device profiles.

**The application is for users.** Adding new devices is the work of developers and contributors, and
their tools do not belong in the interface: no firmware or package analysis, no protocol labs, no
"research" screens. Static package analysis lives only in the developer command `--inspect` and in
`tools/`; the interface's "Diagnostics" only checks the connection and saves a passive report.

**Found devices are shown as square tiles with a 3D model of the device** (lit, turning slightly
toward the pointer, rising on hover with a soft glow behind it), for every device and kind, never as
long flat rows. A device without a model shows its kind icon on the same stage. Rules:
`docs/DESIGN.md`, section 8.1.

**The product is called "Lusorium Studio"** (renamed from the working name "AJAZZ Connect" on
2026-10-28; the exe is `lusorium-studio.exe`). Rules:
- The product name, data paths and strings are never hard-coded as literals. The single source is the
  application identity module (`src/app/identity.rs`): name, data directory, lock prefix, OpenRGB
  client name; the package and exe name live in `Cargo.toml`, the app icon in
  `src/ui/icons/app_icon.rs`. New code refers to the product only through them.
- The product name, icon and UI contain no third-party trademarks. Manufacturer names are allowed only
  as a description of supported hardware ("compatible with ...").
- Compatibility with a manufacturer's own software (for example the AJAZZ Studio mutex) is a detail of
  that brand's driver, not of the core.
- On renaming, data from the old directory is migrated so backups are not lost: the previous name goes
  to `LEGACY_DATA_DIRS` in `identity.rs` (today `%LOCALAPPDATA%\AjazzConnect` moves to
  `%LOCALAPPDATA%\LusoriumStudio` on first launch). The places that carry the name are listed in
  `docs/ARCHITECTURE.md`, section 7.

Only keyboards are implemented today (AJAZZ AK820 MAX and OpenRGB-compatible ones). That is the initial
support, not the boundary of the project.

## Development devices

Two developers, **two different keyboards of the same model**. The main requirement is to never harm
either of them (or any other device).

| Label | Keyboard | Platform | Support status |
| --- | --- | --- | --- |
| Magnetic (co-author) | AK820 MAX Ultra, marking `…ULTRARSW…` | SONiX `0C45`, AA55 protocol | AA55/Feature adapters exist; code was written for it |
| Mechanical (repository owner) | AK820 MAX, marking `…RGPWB2401` (`GPWB2401`) | BY Tech `258A:010C`, radio `3554:FA09` | `by_tech` driver (experimental, wired): protocol restored statically from the vendor package, **writes not yet verified by the owner** |

These are **different controllers and different protocols**. The name "AK820MAX" proves nothing; the
same holds for any brand: one name does not mean one protocol. Full data, versions, package hashes and
the verification log: `docs/DEVICES.md`.

## Hard code rules

1. **Rust only** (edition 2024). Python in `tools/` consists of standalone analysis labs; it is not part
   of the application and never opens HID. No new functionality in other languages.
2. **Structure beats speed of writing.** One module, one responsibility. Target directory layout and
   file-splitting rules: `docs/ARCHITECTURE.md`. Soft file limit is about 300 lines; beyond that, split
   instead of gluing things together.
3. **OOP by Rust means**: data and behavior together (`struct` + `impl`), private fields, public
   methods that protect invariants; polymorphism through `trait`, not through `match` over a driver
   enum; composition instead of "inheritance". A new device must not require edits to the UI or to
   other drivers.
4. **No junk code**: no dead code, commented-out blocks, "just in case" functions, copy-paste,
   `unwrap()`/`expect()` on input data, or `unsafe` outside `transport/` (and every `unsafe` carries a
   `// SAFETY:` comment). Compiler and clippy warnings are errors.
5. **Memory and CPU.** The app is built for gamers: in the background it practically does not exist.
   Rules and budgets are in `docs/ARCHITECTURE.md`, section "Performance". In short: no permanent loops
   or polling, repaint only on events, a HID descriptor is read at most once per session, no idle threads.
6. **Extensibility and universality**: the core, the UI and `app/` know nothing about brands or device
   types; they work through capabilities (lighting, keys, DPI, fan speed, ...). A community driver is
   added as one folder in `src/drivers/` plus one line in the registry; a new device of an already known
   protocol is added only as a profile data entry, with no code. Contract and checklist:
   `docs/ADDING-A-DRIVER.md`.
7. **The interface is alive and smooth**: everything is interactive, and states and page transitions are
   animated physically (springs, inertia, interruptibility), but animation spends resources only while
   something is moving. Rules: `docs/DESIGN.md`, section 9.
   **The layout is adaptive**: it fills the window at any size and reflows while the window is resized
   (columns on wide windows, stacking on narrow ones, breakpoints only from tokens); no fixed page
   width, only running text is capped. Rules: `docs/DESIGN.md`, section 5.1.
8. **Language.** Code identifiers, code comments, commit messages and all documents in `docs/` are in
   **English**. User-facing UI text lives in a single string table (`ui/strings.rs`, target) so it can
   be translated; the Russian UI is the current default. Comments explain "why", not "what". Older
   research notes in `docs/` (`KEYBOARD-FAMILIES-RE.md`, `ORIGINAL-UPDATER-RE.md`,
   `DRV-COMPATIBILITY-AUDIT.md`) are in Russian and are not being translated for now. `README.ru.md`
   is the Russian README.

## Hard hardware-safety rules

Violating any item is a reason to revert the change, even if "everything worked".

1. **Identification is passive and read-only.** Device enumeration, descriptors, `bcdDevice`, report
   sizes. Commands are sent to a device only after its family is established *exactly*: VID/PID + usage
   page + report sizes + a reply to a safe request.
2. **An adapter for one platform must not accept a device of another.** Every new adapter must have a
   test: the probe profiles of all known devices (from `tests/fixtures/`, list in `docs/DEVICES.md`)
   are rejected by a foreign adapter.
3. **A write that persists in device memory (flash/EEPROM) happens only on an explicit user button
   press**, after a backup of the original state and with a read-back. Streaming color writes stay off
   until it is proven they do not wear the memory.
4. **Firmware, bootloader (ISP), erase, calibration and reset** are not implemented and not called. An
   exception is possible only after an entry in `docs/DEVICES.md`: board, controller, exact image,
   recovery path and the written consent of the owner of *that* device.
5. **Someone else's hardware is verified by its owner.** The result goes to the log in
   `docs/DEVICES.md`. Codec tests and reference packages do not replace a check on the device.
6. **Vendor code is never run** (`OemDrv.exe`, installers, updaters). Archives are unpacked only into the
   ignored directories `research/downloads/` and `research/drv-audit/` and studied statically.
7. **Never copy commands between families "by similarity".** A similar report size or a similar opcode is
   a hypothesis, not proof.
8. **Privacy**: serial numbers, Windows device paths, key presses and personal files must not end up in
   JSON reports, commits or documentation.

## Commands

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

Passive diagnostics (writes nothing to the device; the EXE is built as a GUI app, so read the result from
the JSON, not from the console):

```powershell
target\release\lusorium-studio.exe --probe report.json
```

`build.ps1` and `package*.ps1` contain hard-coded paths of the previous developer
(`C:\Users\pavel\...`). Do not use them for a regular build; plain `cargo` is enough.

## Documentation map

| File | Purpose |
| --- | --- |
| `docs/ARCHITECTURE.md` | Code structure now and the target structure, file-splitting rules, performance |
| `docs/ADDING-A-DRIVER.md` | How to add your own driver or device: contract, checklist, tests |
| `docs/DESIGN.md` | Design system: WinUI 3, colors, grid, icons, acrylic, motion |
| `docs/DEVICES.md` | Both keyboards, USB identifiers, driver packages, verification log |
| `docs/KEYBOARD-FAMILIES-RE.md` | Protocol family research (Russian) |
| `docs/DRV-COMPATIBILITY-AUDIT.md` | Audit of the packages from `Downloads/Drv` (Russian) |
| `docs/ORIGINAL-UPDATER-RE.md` | Analysis of the Chinese SONiX flasher (Russian) |
| `research/SOURCE-MANIFEST.json` | Local copies of external sources and their SHA256 |

## Definition of Done

- [ ] `fmt`, `clippy -D warnings`, `test` pass; `release` builds.
- [ ] New code follows `docs/ARCHITECTURE.md`; no "dump" files.
- [ ] A driver has a codec on plain bytes and hardware-free tests on fixtures.
- [ ] Verified that the adapter rejects a keyboard of another family.
- [ ] The UI follows `docs/DESIGN.md` (colors only from tokens, icons only from the set).
- [ ] The layout is adaptive (section 5.1 of `DESIGN.md`): checked with `--review` at 720, 1040 and
      1600 px.
- [ ] Every action and state change is smoothly animated with springs from `ui/motion` (section 9 of
      `DESIGN.md`): interruptible, velocity-preserving, no endless animations; no repaint when idle.
- [ ] Documents are updated: `docs/DEVICES.md` (if support changed), `README.md`, `STATUS.json`.
- [ ] No serial numbers, vendor binaries or user data in the commit.
