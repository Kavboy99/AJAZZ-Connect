# Keyboard input-loss incident and corrective actions

## Subsequent owner authorization for explicit manual verification

The owner reports the bug corrected and explicitly requests unblocking hardware testing. The blanket
ordinary-apply quarantine below is historical: the current implementation permits only explicit
Select/Apply on the identified tri-mode revision, using fresh reads, a backup before writes,
separately verified palette/configuration commits and no retry. Card selection and sliders never
write. See DEVICES.md, Current manual lighting verification, for the current contract and remaining
physical-verification limits. Recovery success and synthetic tests alone are not treated as proof
of firmware timing or as permission for automatic persistent writes.


## Confirmed outcome

The owner's mechanical GPWB2401 stopped producing key input during lighting editing. Encoder
rotation and illumination continued. Power cycling did not recover keys. The owner independently
ran the official application's reset; lighting changed, but key input did not return. Fn still
activated a radio-mode indicator. These observations do not establish a dead MCU or erased firmware.

The application read the same wired identity (`258A:010C`, release `0100`, password
`03 00 00 00 01 5E`). Its 128-byte configuration was invalid: effect byte `0x0A` was `FF` and
several lighting fields were outside their known ranges. Forty-five bytes differed from the latest
pre-incident backup. The first part of the configuration contained RGB-like patterns in unrelated
fields. After the owner's vendor reset, lighting fields parsed again (effect 11), while 53 bytes
still differed from the earliest original backup, including system fields.

With explicit owner authorization and a direct Yes button press, the recovery tool preserved the
current block, sent one original 128-byte configuration (`0x04`), waited 500 ms and read it back.
The entire block matched. The owner immediately confirmed that key input worked again. No palette,
key-map, reset, calibration, firmware or bootloader write was needed for this recovery.

## What the chat history and source establish

The chat **Исправить режимы подсветки** (2026-10-29) first retained explicit Apply because the
existing protocol writes persistent memory. A subsequent explicit request for immediate hardware
application was implemented using that same persistent path, with a latest-value queue and a
200 ms minimum dispatch interval. The implementation did not establish a volatile command or
flash-endurance guarantee. A request for immediate UX did not supply that missing protocol evidence;
the implementation should have kept the safety boundary and used immediate local preview.

The queue dispatched one operation per device, coalesced edits while busy and cancelled pending
work after an error. The source does **not** demonstrate simultaneous lighting workers racing
against each other. It could nevertheless execute many persistent exchanges while a slider moved.
The old path wrote a changed configuration, waited only 30 ms, then wrote a changed palette. Its
200 ms storage delay occurred after the writes, not between the two blocks. The adequacy of that
interval for this firmware was never verified.

Reply validation previously checked report ID, opcode and minimum length for configuration/palette,
but not all header fields. The fake device stored writes synchronously; passing its tests did not
exercise firmware storage timing, partial persistence or real input scanning. Neither the number
of writes in the incident nor their individual timings were captured in a live packet trace.

## Cause and remaining uncertainty

The immediate recoverable failure was a corrupted system configuration. Returning the original
block restored input. The confirmed software defect was allowing automatic persistent configuration
writes in a UI edit queue without evidence of safe streaming. The unsafe configuration-plus-palette
write path was reachable during ordinary color edits.

It is **not proven** that write frequency alone caused corruption. Palette handling, firmware storage
timing, interleaving with other software and another protocol assumption remain possible contributors.
Static reinspection of this exact vendor binary reconfirmed configuration command `0x04`, palette
command `0x0A`, Report ID 6, 520-byte frames and the documented byte layout; it did not establish a
safe live-edit rate. No firmware-wear or permanent physical-damage conclusion follows from the data.
Do not present a more specific firmware mechanism as established without new evidence.

## Implemented containment and recovery

- The automatic edit queue rejects all persistent capabilities. Immediate preview remains local;
  other persistent drivers retain an explicit Apply button.
- Ordinary BY Tech lighting apply/restore implementations now reject before any HID exchange. The
  previous configuration-plus-palette write routine has been removed from production code.
- The palette packet builder itself refuses writes, including from a future accidental caller.
- Every known BY Tech read validates the entire eight-byte header and bounds the returned length.
- The separate `ConfigurationBackup` capability makes backup/recovery independent of lighting.
  Discovery retains a correctly identified device even if its current lighting configuration is
  invalid, so the recovery controls remain reachable.
- Owner-verified tri-mode release `0100` recovery requires the exact captured descriptor, current
  password and a snapshot belonging to the same hashed endpoint/protocol. It stores the current raw
  blocks first, writes only the original system configuration once and checks all 128 bytes after
  500 ms. It never restores the archived palette. Other revisions cannot use this recovery path.
- Compressed archives are size bounded and include a payload SHA256. Foreign, invalid, oversized,
  truncated or checksum-mismatched archives are rejected. Legacy JSON backups remain importable.
- Both applications require a direct final confirmation for restore. Opening a page, choosing an
  archive or changing a visual preview cannot restore a device.

The SHA256 detects accidental archive corruption; it is not a signature proving an archive's author.
Device/protocol checks and driver validation remain necessary. No archive is executed or extracted.

## Evidence retained locally

Ignored `verification/input-loss-probe.json` and `input-loss-after-vendor-reset.json` retain the
read-only reports. Ignored `research/drv-audit/input-recovery/` retains the original backup, exact
recovery plan, Rust source and before/after blocks. The later optional probe without wired
identification is not additional verification. The successful immediate read-back and the owner's
physical confirmation establish recovery. Personal identifiers, OS paths and key presses are not
copied into this report.

The original persistent-write feature, not the new general backup UI, was exercised during the
incident. The new UI and archive flow are verified with synthetic devices; no new overnight hardware
write or full-memory-dump claim is made. Complete key-map/macro/display backup and arbitrary keyboard
support require independently identified read/write protocols and owner verification.
