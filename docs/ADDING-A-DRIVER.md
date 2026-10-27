# How to add a driver or a device

A guide for community contributors. The application is meant for devices of any manufacturer and kind,
so there are two ways to add support.

**Path A: a new device of a known protocol (no code).** If the protocol is already implemented by a
driver (for example `hid_lamparray` or `sonix_aa55`), a record in
`src/profiles/data/<vendor>.toml` is enough: name, manufacturer, device kind, VID/PID, driver,
capabilities. A profile only hints which driver to choose; admission to writes is checked by the driver
itself. Add a probe fixture and an entry in `docs/DEVICES.md`.

**Path B: a new protocol (a driver).** A driver is a folder in `src/drivers/` (named after the protocol,
not the brand) and one line in the registry. The contract is described in `docs/ARCHITECTURE.md`,
section 3. It is a **target**: the code still uses `enum Session`. Write new drivers directly against
the `Driver` and `Device` traits; if `core/` has not been extracted yet, start with step 5 of the
migration and submit it as a separate change, and the driver as the next one.

## Before you start: prove it is your device

1. Take a passive report: `ak820max-assistant.exe --probe report.json`. It writes nothing to the device.
2. Record VID/PID, `bcdDevice`, usage page, Input/Output/Feature report sizes and descriptor hashes. Do
   not paste the serial number or the Windows path into discussions.
3. Find a source for the protocol: the official driver (static analysis, never run it), a licensed open
   project, or your own measurements. Record the link and SHA256 in `research/SOURCE-MANIFEST.json`. Code
   from GPL projects is not copied: implement from a description of the format.
4. Check `docs/DEVICES.md`: there may already be an entry for this family.

## Folder layout

```
src/drivers/<name>/
  mod.rs        public driver type and re-export; nothing else
  codec.rs      pure functions: build a packet, parse a reply (no IO)
  driver.rs     impl Driver / impl Device: request order, pauses, ACK checks
  backup.rs     (if rollback exists) backup format and validation
  tests.rs      codec and failure tests, fixtures in tests/fixtures/
```

Files stay under ~300 lines, with no `utils.rs`. Whatever is not needed outside is private.

## Capabilities, not brands

A driver declares what a device can do with separate capability traits (`Lighting`, `KeyRemap`, `Dpi`,
...; `docs/ARCHITECTURE.md`, section 3). If the device only does lighting, implement only `Lighting`. A
new device kind (for example a fan) starts with a new capability in `core/capability.rs` and a page in
`ui/pages/`; existing drivers do not change. Anything brand-specific (locks of the vendor software,
firmware-specific fixes) stays inside your driver and does not reach `core/`, `app/` or `ui/`.

## What a driver must guarantee

1. **`discover()` only reads.** Enumeration and descriptors are allowed. The first write command comes
   only from `apply_*`, triggered by a user action.
2. **Strict admission.** A VID/PID match is not enough: check the usage page, report sizes and a safe
   reply from the device. An unknown structure means refusal, not a guess.
3. **A foreign device is rejected.** Test: the probe profiles of all project devices
   (`tests/fixtures/probe-*.json`) must not be accepted by a foreign driver.
4. **A write is not repeated after a lost reply.** If the packet may have arrived, a retry is dangerous.
5. **Persistent settings (flash/EEPROM) are flagged** in `LightingTraits` (or the equivalent for another
   capability). `snapshot()` is taken before the first write and a read-back follows. If a snapshot cannot
   be taken, the driver explicitly declares "no rollback" and the UI shows it before applying.
6. **No firmware, bootloader, erase, calibration or reset.** These are outside the contract.
7. **Errors are typed** (`DriverError`), with no `unwrap()` on device data.
8. **Memory and CPU**: no background threads, the handle is closed after the operation
   (`docs/ARCHITECTURE.md`, section 5).

## Registration

In `src/drivers/mod.rs` add a feature flag to `Cargo.toml` and a line in `registry()`:

```rust
#[cfg(feature = "driver-<name>")] Box::new(<name>::<Name>Driver::new()),
```

The UI, `app/` and other drivers need no changes. If you had to change them, the contract is
insufficient: open a discussion instead of working around it.

## Tests

- Codec: building and parsing packets on fixtures; errors (truncated data, foreign echo, wrong length).
- Admission: a matching device is accepted, neighboring families are rejected.
- Rollback: a backup is checked for belonging to the device.
- A hardware test is `#[ignore]` and is run by the device owner; the result goes into the log in
  `docs/DEVICES.md`.

## Change checklist

- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test` are green.
- [ ] The protocol source and SHA256 are recorded in `research/SOURCE-MANIFEST.json`.
- [ ] Entry in `docs/DEVICES.md`: device, identifiers, status (`identified` → `read` →
      `written by owner` → `confirmed`).
- [ ] Licenses of dependencies and sources are in `THIRD-PARTY.md`.
- [ ] The change description states who verified it and on which device.
