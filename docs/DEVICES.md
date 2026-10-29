# Devices and compatibility

## Lighting follow-up and release delivery, 2026-10-29

The owner clarified that the inverted controls affect the physical keyboard only; the 3D preview
is correct. They also report action buttons blinking at the five-second battery refresh interval.
The ordinary `target/release` executable was built at 17:32, before the field and telemetry fixes;
the prior corrected release was delivered only under `target/panel-review/release` at 18:30.
This follow-up rebuilds the ordinary release path with the corrected codec. For Breathing (hardware
id 2), brightness occupies 0x3C and speed the high nibble of 0x3D. Independent-byte regression checks
cover both id 2 and id 7, including preservation of every unrelated byte and the color slot.

A hardware-free test now renders the complete inspector and cards over three battery transactions:
the painted shapes are identical before, during and after each read. One Apply click during telemetry
is retained and dispatched after device release. Progress uses a fixed icon slot and a spring fade,
and only the clicked action shows progress. The owner clarified that Select must remain on a
highlighted preview and disappear only for the confirmed hardware mode; the earlier preview-based
hiding was reverted. Select and Apply use the existing checkmark icon. These tests and the release
rebuild do not establish a new physical result;
the owner still needs to compare brightness and speed using the updated ordinary release.

## Owner-confirmed lighting field correction, 2026-10-29

Full independent re-audit: [Lighting protocol and color audit](LIGHTING-PROTOCOL-COLOR-AUDIT-2026-10-29.md). A fresh download reproduced the pinned executable hash; 19 static instruction anchors passed. Existing snapshots confirm intermediate RGB storage. Neither memory read-back nor the revised HSV conversion proves the optical transfer function or PWM depth.

The owner reports that the Brightness control changed animation speed. Static reinspection of the
same vendor executable confirms the inversion in our former decoder/writer. Each effect pair at
`0x38 + 2 * hardware_id` is `[brightness, (speed << 4) | color_slot]`, not the former opposite
interpretation. `SpeedHW` is loaded into the table at object offset `2E50` (xref `410CE6`),
and `LightHW` into `2E70` (xref `410E00`). The writer at `48CE9C..48CEC4` stores the
LightHW value in the first byte and shifts the SpeedHW value into the second byte's high nibble.
The owner observation confirms the distinction physically; the new build still needs an owner
comparison. Native metadata, configuration validation, writes and passive report fields now agree.
Historical brightness/speed labels in old reports and notes were decoded in reverse and must not be
used as an independent protocol oracle. Regression tests assert the individual changed offsets and
preservation of the other field and color slot, rather than only encode/decode round-trips.

The vendor copies arbitrary RGB channel bytes into the palette (`48D780..48D7B4`); static evidence
does not establish a restriction to white and saturated colors. The picker formerly gamma-encoded
HSV output: 50% cyan saturation produced 188/255/255 instead of the numeric-channel 128/255/255.
The wheel, swatch and transmitted RGB now use the same direct channel conversion without that extra
encoding. Actual LED transfer response and optical color matching remain unmeasured. Saturation
and white at 0% remain available; do not claim a firmware quantization limit from washed-out output.

## Current manual lighting verification, 2026-10-29

The owner states that the input-loss bug has been corrected and explicitly requests removing the
blanket write block so hardware testing can continue. Manual lighting commits are enabled for the
identified tri-mode password/release 03000000015e-0100 with valid cached configuration and palette.
Discovery, card selection and sliders remain read-only/local preview; only Select or Apply starts a
transaction. This authorization supersedes the ordinary-apply quarantine described in the historical
incident entries below. Firmware/reset/calibration and automatic persistent writes remain absent.

Each explicit commit rechecks the family password and reads fresh blocks, validates the draft and
stores both original blocks before any write. Changed palette bytes are written and read back first;
only then may the configuration be selected, after checking that it has not changed concurrently.
Each written block gets 500 ms before read-back; final palette validation catches cross-block changes.
There is no retry or automatic rollback. The cached applied mode changes only on full success.
The new path has synthetic failure/backup/identity/framing tests; this change itself does not claim a
new physical-device verification. The delay is a conservative implementation choice, not a measured
firmware guarantee. Independent Settings backup/recovery remains available; ordinary lighting restore
is not exposed. Historical evidence and the recovered input state are retained below.

## Historical safety incident: mechanical GPWB2401, 2026-10-29

The owner reported complete loss of key input during lighting editing. Disconnecting and reconnecting
did not restore input; illumination and the encoder still work, but key-based lighting controls do not.
A read-only USB probe still confirms `258A:010C`, release `0100`, password `03000000015e`.
The returned 128-byte configuration fails validation: effect byte `0x0A` is `FF`, and several
speed/brightness fields are invalid. Forty-five bytes differ from the latest pre-incident backup.
The configuration contains RGB-like bytes in unrelated fields; the exact corruption mechanism is
not established. Do not infer a firmware erase or hardware failure from this observation.

Both pre-incident backups are retained locally. At incident containment, all application BY Tech lighting writes, including
restore, were quarantined before any exchange. Manual Apply is now re-enabled under the current authorization above. The native preview remains available. Persistent
capabilities cannot enter the automatic edit queue; other persistent drivers require explicit Apply.
This supersedes the live persistent-write behavior described in the historical entries below.

A separate local Rust recovery tool is prepared under ignored `research/drv-audit/input-recovery/`.
It defaults to an offline plan; the write option requires explicit owner authorization and a direct Yes button press in a native confirmation dialog. It verifies
the original hashed endpoint identity, exact descriptor, release, password and diagnosed damaged block,
then saves that block and can send one `0x04` configuration write from the earliest original backup.
It never writes the palette, key matrix, firmware, reset or bootloader. Read-back is mandatory and
there is no retry. The recovery was subsequently executed with a direct owner confirmation; see the recovery result below.

A registry of explicitly supported devices and hardware revisions, their direct protocol adapters and verification evidence. Current development targets are the two keyboards below. Update this file after every new verification. Statuses: `identified` → `read` → `written by owner` → `confirmed`. Anything not verified
on the device is marked as an inference, not as a fact.

**Serial numbers are not entered into the repository.** On the AJAZZ website the driver is matched by the
full name and the last six digits of the barcode; each owner keeps that data to themselves. Only model
markings are given here.

## 1. The project's two keyboards

One model by name, different platforms.

| | Magnetic (co-author) | Mechanical (repository owner) |
| --- | --- | --- |
| Marking in the serial number | `…SGAK820MAXULTRARSW…` | `…SGAK820MAXRGPWB2401` (`GPWB2401`) |
| Type | AK820 MAX Ultra, magnetic switches | AK820 MAX, mechanical switches, three connection modes |
| Platform | SONiX (`0C45`) | BY Tech (`258A`), Compx radio (`3554`) |
| Status | Identified from project materials. `--probe` has not been run on this PC | **Identified by `--probe` on 2026-10-28**; protocol restored statically, writes not yet verified |
| Manufacturer driver | `AJAZZ_AK820_MAX_Single_Mode_RGB_Magnetic_Axis_Keyboard_Driver_Installer` V2.06.01 | `AJAZZ_AK820_MAX_driver_V1.0` (BYCOMBO4) |
| What the application can do | Color via AA55 (main), SONiX Feature (experimental), diagnostics | Solid color via `by_tech` (experimental, wired only; **not yet verified on the keyboard**), diagnostics |
| Firmware | The SN34F280 ISP model is described, **writing is not implemented** | Neither the controller nor the image is known |

Earlier, the README and `STATUS.json` wrongly attributed the `GPWB2401` marking to the co-author's
keyboard. Fixed on 2026-10-28: `GPWB2401` is the repository owner's keyboard.

### 1.1 Mechanical AK820 MAX (`GPWB2401`)

Captured by a passive `--probe` on Windows 11, wired connection, 2026-10-28. Fixture:
`tests/fixtures/probe-by-tech-gpwb2401-wired.json` (without the serial number and paths).

| Parameter | Value |
| --- | --- |
| USB (wired) | `258A:010C`, string "Gaming Keyboard", `bcdDevice` 0100 |
| USB (2.4G receiver) | `3554:FA09`, string "2.4G Wireless Receiver", `bcdDevice` 2008 |
| Modes | USB, 2.4G, Bluetooth (per the V1.0 package, the Thr branch) |
| Windows driver | Stock (`usb.inf` verified on the receiver); the application needs no vendor driver |
| Match with the V1.0 package | Wired VID/PID `258A:010C` and radio `3554:FA09` are listed in the package's `Cfg.ini`/`KB.ini` |
| SONiX signs | None: no `0C45`, no usage page `FF68`, no AA55 |
| LampArray (`0059:0001`) | None |

Interfaces on the wired connection (usage page : usage, Input / Output / Feature in bytes):

| Purpose | Usage | In / Out / Feat | Descriptor SHA256 |
| --- | --- | --- | --- |
| Keyboard (boot) | `0001:0006` | 9 / 2 / 0 | not captured |
| System keys | `0001:0080` | 2 / 0 / 0 | not captured |
| Consumer | `000C:0001` | 3 / 0 / 0 | not captured |
| Vendor (input) | `FF00:0001` | 4 / 0 / 0 | `5fde8a59…2c0f91` |
| Keyboard (NKRO) | `0001:0006` | 16 / 0 / 0 | not captured |
| Vendor (feature 6) | `FF00:0001` | 0 / 0 / 6 | `8548f7a5…ed4dd3` |
| **Vendor (feature 520, Report ID 6)** | `FF00:0001` | 8 / 0 / 520 | `b028c012…fbb1258` |
| Mouse | `0001:0002` | 8 / 0 / 0 | not captured |

Full hashes are in the fixture. The `FF00:0001` channel exposes a 520-byte Feature report with Report ID 6. Report size alone does not establish command compatibility; commands from other controllers must not be copied without protocol verification.

The `3554:FA09` radio receiver: vendor interfaces `FF02:0002` (20 / 20 / 0, SHA256 `63c9063b…9c0406`)
and `FF04:0002` (feature 8, SHA256 `3c311f5d…a2cbb8`). It shows only the radio channel and says nothing
about the keyboard's board. The wired protocol does not apply to the radio.

The owner's PC also has another receiver, `3554:F52D`: by all signs it is a mouse receiver, unrelated to
the keyboard, and ignored in reports.

#### 1.1.1 Configuration protocol (static analysis, 2026-10-28)

Source: `OemDrv.exe` from `AJAZZ_AK820_MAX_driver_V1.0` (SHA256 `fc7109b0…4231c06c`), disassembled
statically, never run. The vendor class for this keyboard is `CDevG5KB`; `KB.ini` (Wired) gives
`Fw=24`, `Psd=3,0,0,0,1,42`, `CRC=1`, `RGBIndex=0,1,2`, `LedOpt1=1,...` (effect 1 "Fixed_on").
Addresses below are virtual addresses in that file, so the analysis can be repeated.

| Fact | Evidence in `OemDrv.exe` |
| --- | --- |
| For `Fw = 24` the vendor opens only the collection with usage page `FF00`, usage 1 and a 520-byte Feature report | `CDevG5KB::FindHIDDevice`, `0x48A8BA`–`0x48A916` |
| Report ID 6 when `Fw = 24` (9 otherwise), 520 bytes per Set/Get Feature | `0x48ACD9`–`0x48ACEB`, `push 208h` before `HidD_SetFeature` |
| Frame: `[06, cmd, sub, 00, pages, page, len_lo, len_hi, data...]`, at most 512 data bytes per page | `CDevG5KB::AccessData`, `0x48ABE0` |
| Read = Set Feature with the request, 20 ms, Get Feature; data from byte 8; the vendor checks neither echo nor checksum (the keyboard does echo the header, see below) | `0x48B3E0`–`0x48B4E0` |
| Family check: request `0x82`, sub 1, 6 bytes; the reply must equal `Psd` byte for byte, else the handle is closed | `GetPassword` `0x48BC10`, compare at `0x48A987` |
| `0x04` / `0x84`: 128-byte configuration block; the vendor itself reads it, changes bytes and writes it back | `0x48A140` (`GetLED` → `[9]=0`, `[0x0A]=effect` → `SetLED`) |
| Block fields: `[0x09]` game-mode flag, `[0x0A]` effect id, from `0x38` two bytes per effect id: brightness, `speed << 4 \| color slot` (slot 7 = random) | `0x48CE20`–`0x48CEDB` |
| `0x0A` / `0x8A`: color table, 21 bytes (7 RGB slots) per effect id from id 1; with `CRC=1` padded to 512 bytes and marked `5A A5` at `0x1FA` | `0x48D759`–`0x48D9C5` |
| Other commands (`0x03` key matrix, `0x05` macros, `0x06` game, `0x0B` screen, `0x08`, `0x0E`) exist and are **not** used | vtable at file offset `0x20E1DC` |

What the `by_tech` driver does with it:
- Admission: the collection above plus the `by_tech` profile (`258A:010C`). A device is accepted only if
  the password reply equals one of the package's `Psd` values: `03 00 00 00 01 42` (single-mode board,
  `Dev/kb/Wired`) or `03 00 00 00 01 5E` (tri-mode board, `Dev/kb/Thr`). The two `KB.ini` files are
  identical in `Fw`, `CRC`, `LedOpt*`, `RGBIndex` and the key layout. Only the USB connection is
  supported; the radio receiver `3554:FA09` is not admitted.
- Replies: Windows reports only the bytes the keyboard sent (8-byte header + data, for example 14 bytes
  for the password), and the keyboard echoes the request header. The driver requires Report ID 6, the
  same command byte and at least the requested data length.
- Discovery and `--probe` send only reads: password, configuration block, color table.
- "Apply" (button press only): reads both blocks, validates effect settings (speed and brightness
  0..4, slot 0..7, table signature `5A A5`), backs up both blocks and selects a native program.
  The editor exposes `LedOpt1..18` (hardware ids 1..18) and Off (`LedOpt19`, hardware id 21).
  Effect names follow `Text/en/text.xml` `tc_kb_led1..18`; the second LedOpt column is the vendor's
  animation selector, not the hardware id. Ids 19/20 are not offered. Brightness and speed use
  `LightHW` / `SpeedHW`; color controls follow the LedOpt flags (no custom color for 3/15/16/17/21).
  The selected effect's saved slot and palette remain intact unless a custom color explicitly
  selects slot 0 and replaces its RGB triple. Other effect settings, key mappings and table bytes
  remain untouched. Only changed blocks are written once; both blocks are read back after 200 ms.
  A mismatch is reported without retry. The legacy solid action retains its former zero-to-four
  brightness behavior; the native editor preserves an explicit zero brightness.
- "Restore" and "From backup" write the saved blocks the same way.

2026-10-28: Native editor integration covered by hardware-free tests (all 19 programs, zero
brightness, color isolation, invalid input refusal, backup ordering, restore and read-back failure).
No lighting write was performed on the owner's keyboard as part of this change; physical verification
is still pending. Wireless lighting remains unavailable.
The receiver-only UI now exposes the driver's native program preview and brightness controls with
an explicit USB/wired-mode requirement and disabled Apply. A passive local probe confirmed a
receiver-only connection when the missing-controls issue was reproduced. Preview metadata never
performs discovery or changes wireless admission or write support.

2026-10-29: The owner connected the tri-mode board by USB and confirmed hardware effect 7 reacts
only to key presses. Read-only configuration/color reads were originally labeled speed 4, brightness 1; the corrected decoding is brightness 4, speed 1, slot 6
(white), with the standard seven-color RGB palette. `--probe` now reports all saved program settings
and palettes after the known password check. Device palette display data is cached alongside the
configuration; opening a color picker no longer turns a displayed device color into a write request.
The known protocol returns settings, not animation frames or firmware algorithms. Existing preview
formulas remain illustrative; this read must not be presented as exact verification of every mode.
Static analysis of `OemDrv.exe` (never executed) located LedOpt parsing at 0x40FA30..0x40FB42 and
localized keyboard lighting labels at 0x42D420..0x42D4CE; it did not establish all animation timing.

**Still unknown** (to be closed by the owner's verification):
- Whether the firmware echoes exactly what was written (the read-back compares whole blocks).
- Whether the flash write of these blocks has a wear limit worth tracking (writes happen only per
  button press).
- Whether 2.4G/Bluetooth use the same blocks: not supported, the receiver is not admitted.
- The controller and bootloader: firmware work stays out of scope.

**Read on the owner's keyboard** (`--probe` over USB, 2026-10-28; reads only):
- Password `03 00 00 00 01 5E`: the tri-mode board, as the keyboard's three connection modes suggest.
- The configuration block matches the layout above byte for byte where the analysis predicted it:
  `[0x09] = 00`, `[0x0A] = 07` (effect 7, "Ripples shining", was active), `[0x0B] = 20` (`StartGp`),
  `[0x38..0x39] = FF FF`, and from `0x3A` pairs such as `04 37` (brightness 4, speed 3, slot 7 =
  random) with every packed speed in 1..4 and every slot in 4..7. The block also ends with `5A A5`
  (`0x7E`). The driver accepted it as valid.

Verification still to do by the owner: with experimental drivers on, one "Apply" and one "Restore" on
the keyboard connected by cable. The result goes to the log below; on success the driver becomes
`Stable` and the profile `written_by_owner`.

### 1.2 Magnetic AK820 MAX Ultra (co-author)

The data comes from project materials, not captured on this PC: `research/keyboard-updater/profiles.json`
(profile `ak820max-ultra-80a0-stock-136`) and `docs/ORIGINAL-UPDATER-RE.md`.

| Parameter | Value |
| --- | --- |
| USB (per materials) | `0C45:80A0`, versions 1.30 / 1.36 |
| Chip | SN34F280, ROM bootloader `0C45:7210` |
| Color protocol | AA55 (SONiX, interface `FF68:0061`, 65/65 reports with Report ID) |
| Other possible PIDs of the family | `80BE`, `8033`, `8009`, `800A` belong to other revisions; do not equate them |
| Radio receiver | `0C45:FEFC` (separate firmware, do not confuse with the keyboard) |

To confirm: ask the co-author to run `--probe` over USB and put the JSON into `tests/fixtures/` as
`probe-sonix-ultra-<description>.json`. Until then the table rows are an inference from project
materials.

## 2. Which adapter applies to which keyboard

Admission matrix. "No" means the adapter rejects the device by VID/PID and interfaces.

| Adapter | Magnetic Ultra | Mechanical GPWB2401 | Admission condition |
| --- | --- | --- | --- |
| SONiX AA55 | Yes (per project materials) | No | `0C45`, `FF68:0061`, Input/Output 65, matching `GET_INFO` |
| SONiX Feature | Possible (experimental) | No | `0C45:8009/800A`, `FF13` Feature 65, `FF68` Output 4097 |
| HID LampArray | No (no such interface) | No | `0059:0001` and a valid descriptor |
| BY Tech | No | Yes, experimental (wired) | `FF00:0001`, Feature 520, profile `258A:010C`, password reply `…01 42` or `…01 5E` (single- or tri-mode board), USB only |

Every "No" in the GPWB2401 column was checked against the code and the probe of 2026-10-28 and is locked
in by `tests/admission.rs` (`docs/ADDING-A-DRIVER.md`, rule 3). Since 0.3.0 the AA55 and Feature
adapters also require a matching entry in `src/profiles/data/sonix.toml`; SONiX Feature is off unless
experimental drivers are enabled. The co-author's keyboard has not been checked with 0.3.0: HID handles
are now opened per operation, so a `--lighting-probe sonix_aa55` and one color write by the owner should
be recorded in the log below before the AA55 row is treated as unchanged.

## 3. Driver packages and sources

Unpacked without running anything (`docs/DRV-COMPATIBILITY-AUDIT.md`, in Russian). Vendor binaries are
not committed to the repository (`research/downloads/` and `research/drv-audit/` are ignored).

| File | Size | SHA256 | Purpose |
| --- | --- | --- | --- |
| `AJAZZ_AK820_MAX_driver_V1.0.zip` | 1 959 226 | `a264ecb2…b0d6cf` | Mechanical: BYCOMBO4, `258A:010C` / `3554:FA09`; `OemDrv.exe` SHA256 `fc7109b0…4231c06c` |
| `AJAZZ_AK820_MAX_Single_Mode_RGB_Magnetic_Axis_Keyboard_Driver_Installer.zip` | 17 983 320 | `6a20f82e…a6de6` | Magnetic: installer V2.06.01 (Inno Setup 5.5.7, not digitally signed), "单模RGB磁轴驱动" |
| `AJ159 APEX-3950 ID：1755.rar` | 22 255 901 | `43e64058…bcb3e53b1` | **Not a keyboard**: mouse and base update utilities (`ry_upgrade`) |

Official catalog for driver matching: https://www.a-jazz.com/en/h-col-159.html (by full name and the last
six digits of the barcode). For the mechanical version the V1.0 package is already in hand; there is no
need to download a Windows driver, because the application uses the stock HID drivers.

### AJ159 APEX display support audit (2026-10-28)

The local RAR above was rechecked against its SHA256. Its two nested ZIPs contain firmware
updaters, not the mouse configuration application. The base's `support_config.json` lists
`3151:5007`, usage page `FFFF`, usage `0002`, interface 2, and a separate boot identity
`3151:402E`. Its `tool_config.json` enables `usb`, `rf` and `oled` upgrade paths. These are
updater metadata, not evidence for safe image-upload commands or a confirmed device identity.

The screen belongs to the charging dock, as described by the
[official product page](https://ajazzstore.com/products/ajazz-aj159-series).
There is currently no AJ159 device profile, native mouse adapter or display-upload capability in
this application. The mouse in `cli/review_state.rs` is a synthetic LampArray fixture for UI review.
Do not present it as AJ159 hardware support. Display customization requires separate protocol work
and verification by the owner; firmware-update payloads must never be used as display commands.

## 4. How to add a device or a verification result

1. `lusorium-studio.exe --probe report.json` (read-only).
2. Make sure the JSON contains no serial number or Windows paths; put the fixture into `tests/fixtures/`.
3. Add a section to this file: identifiers, interfaces, protocol source, what is unknown.
4. Add a row to the log below. For a write to a device: who did it and on whose hardware.

## 5. Verification log

| Date | What | Device | Who | Result |
| --- | --- | --- | --- | --- |
| 2026-10-28 | Automatic search in the application with experimental drivers on (password, configuration block, color table; reads only) | GPWB2401 | owner | Keyboard found as AK820 MAX; the color table passed its `5A A5` check; no writes |
| 2026-10-28 | `--probe` over USB with `by_tech` (password, configuration block; reads only) | GPWB2401 | owner | Password `…01 5E` (tri-mode); block valid, layout confirmed; no writes |
| 2026-10-28 | Static analysis of `OemDrv.exe` (V1.0), protocol of section 1.1.1 | GPWB2401 | audit | Driver `by_tech` written; nothing sent to the keyboard |
| 2026-10-28 | `--probe` over 2.4G (receiver) | GPWB2401 | owner | Only `3554:FA09` is visible; the board is not |
| 2026-10-28 | `--probe` over USB | GPWB2401 | owner | `258A:010C`; no SONiX/AA55/LampArray; read-only, no writes |
| 2026-10-27 | Static check of the Drv packages | n/a | audit | See `docs/DRV-COMPATIBILITY-AUDIT.md` |
| n/a | `--probe` over USB | Magnetic Ultra | co-author | Not done |
| n/a | AA55 color write | Magnetic Ultra | co-author | Not recorded in the repository |

### Battery display (2026-10-28)

The device header exposes AA55 `GET_INFO` byte 17, already parsed and range-checked during discovery,
as a percentage snapshot. This adds no requests and no polling. It is labelled as the value from the
last discovery, not live telemetry. Charging-byte semantics are not inferred. Other drivers display
unavailable battery data. UI/fixture verification is hardware-free; no new physical validation is claimed.

## AJ159 offline placeholder (2026-10-28)

AJAZZ AJ159 currently has an offline catalog tile and model/passport preview only. The user supplied
`assets/models/aj159.glb` (mouse and dock). No controller, VID/PID, report shape, capabilities or protocol
has been established. There is no driver, discovery rule, live session or hardware verification for
this entry. DPI, button assignment and lighting controls are intentionally absent. Enable Developer
mode and Show all supported devices in Settings to view it.

### Mechanical receiver battery investigation (2026-10-28)

The initial diagnostic snapshot listed the owner's `3554:FA09` receiver but did not list
`258A:010C`. This was not evidence of the keyboard's active connection mode. The `FF04:0002` collection matched the recorded 8-byte Feature descriptor
(Report ID 6) exactly. A standalone local Rust diagnostic performed one GET_FEATURE on that
collection, with no SetFeature, output report, keyboard-input read, or settings write. The receiver
returned eight bytes. Their meaning is unverified; no battery percentage or charging state can be
reported from this result. A byte whose numeric value is 100 is not evidence of 100 percent charge.
Raw results and the diagnostic are retained only in ignored `research/` files.

The exact V1.0 package sets `ShowPower=0` in both `Dev/kb/Wired/KB.ini` and
`Dev/kb/Thr/KB.ini`. This disables the vendor battery display; it does not prove that the hardware
cannot expose battery data. Static analysis also found a routine with the error string `GetPower
failed` at `0x48FEA0` (string reference `0x4900CB`), but it uses 520-byte reports with Report ID 9
and a SetFeature/GetFeature exchange. That is not the receiver's 8-byte Feature collection, and this
request was not sent. A verified decoder or a confirmed read-only receiver query is still required.

Correction after the owner's connection clarification: Windows PnP and a repeated live probe both
confirm the wired `258A:010C` keyboard alongside the receiver. The wired 520-byte Feature collection
successfully answered the existing family-password and configuration read requests: the tri-mode
password matched and the configuration parsed successfully. This confirms wired communication, not
battery telemetry. The earlier receiver result must not be attributed to the wired keyboard or used
to infer its active connection mode. Why the earlier snapshot omitted the wired device is unknown.

### Wired battery read verification (2026-10-28)

A standalone Rust check admitted only `258A:010C`, `FF00:0001` collections with exact descriptor
matches to the owner's recorded Feature interfaces, and issued one passive GET_FEATURE per matching
interface, without SetFeature or output writes. Report ID 5 (6-byte Feature) failed to return a report.
Report ID 6 (520-byte Feature) returned 136 bytes with the header of the previous configuration read
(`0x84`, 128 data bytes). This result is configuration data, not a verified battery value. No battery
percentage was obtained. These observations do not prove that every possible USB battery query is
unsupported.

Static dispatch verification in the exact V1.0 vendor executable: the `CDevG5KB` vtable at VA
`0x60FBDC` (file offset `0x20E1DC`) uses slot 8 for the battery getter, pointing to `0x485500`:
`xor eax, eax; ret 4` (failure stub, no I/O). In the other class's vtable at `0x6105AC`, slot 8 points
to the real `GetPower` routine at `0x48FEA0` (reference at `0x6105CC`). That routine must not be
transplanted into the owner's driver. Together with `ShowPower=0`, this establishes that the exact
vendor class does not implement battery reading; it is not proof of a firmware limitation.

### Wired battery query found and read (2026-10-28)

Supersedes the earlier inability to obtain battery telemetry: a standalone Rust diagnostic successfully
read the owner's mechanical `GPWB2401` battery query over the wired `258A:010C` connection. The app's
`by_tech` driver and UI have not yet been changed to issue this query.

Protocol source: [AulaSofwareLinux `src/proto.rs`](https://github.com/leisiji/AulaSofwareLinux/blob/main/src/proto.rs),
Git blob SHA `0ef893d0bdbbf5ed042128b8ec8dd16476669a27`. It describes the same BYCOMBO4 / `CDevG5KB`
Feature framing, `GET_POWER=0x87`, parameter 2, two reply bytes, with the first interpreted as the
battery percentage. This supplied a same-family read candidate, not proof of support on this board;
the following live exchange established that the owner's firmware answers it.

Admission was restricted to VID/PID `258A:010C`, usage `FF00:0001`, the exact recorded 520-byte Feature
descriptor (Report ID 6), then a fresh password reply equal to `03 00 00 00 01 5E`. After that check,
the tool sent exactly one zero-padded 520-byte read request beginning `06 87 02 00 01 00 02 00`.
The keyboard returned 10 bytes: `06 87 02 00 01 00 02 00 64 10`. The full eight-byte header matched,
so this was not the stale `0x84` configuration reply seen during passive reading. The battery field
was 100 percent at this read; the second byte `0x10` remains an uninterpreted status byte. No charging
flag or measurement accuracy has been verified, and no continuous monitoring was performed.

No configuration, firmware, reset, calibration or bootloader command was sent. The diagnostic allows
only the fixed identity and battery read tuples, with no retry; its tests reject other opcodes,
parameters, lengths, truncated replies and stale configuration headers. Its two tests and clippy with
warnings denied passed. Source and local raw results are retained in ignored `research/drv-audit/`.

Additional research clarified the earlier static result: the real native `GetPower` routine belongs
to `CDevG5MS` (mouse), not `CDevG5KB` (keyboard). The keyboard's vendor getter remains a stub, but the
live firmware supports the read anyway. Public Womier L65 notes also describe battery access on
related boards; their radio `0x4A` must not be assumed equivalent here: in this exact vendor package
`CDev3632::GetOnBoard` at `0x486140` uses `0x4A` for another purpose. No radio query was sent.

### Radio battery read verification (2026-10-28)

After the owner disconnected the USB cable and selected radio, Windows PnP listed `3554:FA09` and
no `258A:010C`. A standalone Rust diagnostic required the wired device to be absent, exactly one
`3554:FA09` vendor collection `FF02:0002`, and an exact match to its recorded 20-byte Input/Output
descriptor (Report ID `0x13`).

Static analysis of this exact vendor executable confirms the read framing in `CDev3632` at
`0x485CC0`: Report ID `0x13`, command, one packet, index zero, operation zero, zero payload,
additive checksum. Its `GetOnBoard` method at `0x486140` issues read `0x4A`, with no settings write.
The name does not itself establish battery semantics. The independent public implementation
[Womier L65 `linux/l65ctl.py`](https://github.com/deepan-alve/womier-l65-linux/blob/main/linux/l65ctl.py)
interprets reply byte 5 as battery percentage; byte 6 is status with undecoded bits. This is a
same-radio-protocol reference, not a claim that the two boards are interchangeable.

The tool sent exactly one request (no retry):
`13 4A 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 5E`.
The reply was:
`13 4A 01 00 02 4E 01 00 00 00 00 00 00 00 00 00 00 00 00 AF`.
Report ID, command, single-packet count/index, declared two-byte payload and checksum matched.
The battery field was 78 percent (`0x4E`), with raw status `0x01`. Charging semantics remain unknown.
No keyboard-input collections were read; no settings, firmware or bootloader operation occurred.
Two codec tests and clippy with warnings denied passed for this local diagnostic.

A subsequent passive read of the receiver's separate `FF04:0002` Feature report still returned the
earlier opaque values containing `0x64`; they did not track the radio battery reply and must not be
used as battery telemetry. Raw results and diagnostic sources are retained in ignored
`research/drv-audit/`.

The preceding wired query returned `64 10` (a percentage-like value of 100), whereas this radio
query returned `4E 01` (78). Therefore the earlier statement that the USB value establishes the
physical battery charge is too strong. Both transport replies are verified, but their discrepancy
is unresolved: USB could expose a placeholder, stale value or different state. A controlled
cross-transport comparison and independent on-device/OS indication are still needed before treating
USB data as an accurate battery measurement. Neither query is integrated into the app yet.

### USB cable attached while the switch remains on radio (2026-10-28)

The owner reported attaching the USB cable without changing the radio-mode switch. A fresh wired
read diagnostic found no matching `258A:010C` collection and sent no request. Windows PnP independently
listed the keyboard receiver `3554:FA09`, with no wired `258A:010C` device present.

One radio `0x4A` read returned
`13 4A 01 00 02 5A 10 00 00 00 00 00 00 00 00 00 00 00 00 CA`.
The frame and additive checksum validated: percentage field 90 (`0x5A`), raw status `0x10`.
Thus the radio query can report a value below 100 even while the cable is attached. The status
changed from the earlier cable-disconnected radio sample (`0x01`) to `0x10`, which is consistent
with a power-state dependency but does not establish the charging-bit definition. These samples
were taken at different times; the rise from 78 to 90 is not a controlled measurement of charge rate.
This test cannot isolate the earlier USB 100 reading because the USB data endpoint is absent in
this switch position. A comparison after switching to USB mode remains necessary. No settings
or firmware writes were performed. Local outputs are retained in ignored `research/drv-audit/`.

### USB switch follow-up after radio reported 90 (2026-10-28)

The owner then switched to USB mode with the cable still attached. Windows PnP now listed wired
`258A:010C` as well as the physically connected receiver. A fresh exact-profile password read
matched `03 00 00 00 01 5E`; the single battery query `0x87`, parameter 2 returned
`06 87 02 00 01 00 02 00 5C 10`. The full request header matched. The percentage field was 92
(`0x5C`) with raw status `0x10`, versus the preceding radio sample of 90 (`5A 10`).

This establishes that the owner's wired battery reply is not permanently fixed at 100 when powered
by USB. The two recent samples are close but not simultaneous; their difference does not establish
charging rate, measurement accuracy, or exact cross-transport equivalence. The reason for the earlier
100-versus-78 discrepancy remains unresolved. Both transports can return a percentage-like value,
and `0x10` status is observed with the cable attached in either switch position. No settings or
firmware writes were performed. The standalone output is retained in ignored `research/drv-audit/`.

### Radio follow-up after removing the cable (2026-10-28)

The owner reported removing the cable and returning to radio. Windows PnP listed only `3554:FA09`
for this keyboard; wired `258A:010C` was absent. One radio `0x4A` query returned
`13 4A 01 00 02 4D 01 00 00 00 00 00 00 00 00 00 00 00 AE`.
The frame and checksum validated. The reported percentage field was 77 (`0x4D`), status `0x01`.

Across the owner's connection changes, both cable-disconnected radio samples returned status
`0x01`; cable-attached radio and USB samples returned `0x10`. This supports interpreting these
observed whole-byte values as battery-powered versus externally powered states for this unit.
It does not distinguish active charging from full-battery external power, and does not establish
individual bit meanings or all possible status values. With the cable physically disconnected,
this observation is consistent with not charging.

The reported percentage changed from USB 92 to cable-disconnected radio 77. The earlier explanation
that the close 90/92 readings demonstrated comparable charge was premature: connection or power
state may affect the reported estimate. These are verified protocol fields, not calibrated physical
state-of-charge measurements. No discharge rate should be inferred from this sequence. No settings
or firmware writes occurred; raw local results remain in ignored `research/drv-audit/`.

### Battery support integrated into the application (2026-10-28)

`by_tech` now exposes a discovery-time battery snapshot in the device header. Enable experimental
drivers in Settings, then refresh device discovery to obtain a new snapshot. There is no polling,
keep-alive, persistent HID handle or battery worker at idle. The existing Refresh action is the
explicit way to update a value without reconnecting.

USB sends one `0x87` / parameter 2 read only after the known tri-mode password and release `0100`
have been confirmed; a battery failure leaves the existing lighting device usable. The battery parser
rejects percentages above 100, short payloads and mismatched full request headers. Single-mode and
unverified releases receive no new battery command.

Radio admission requires profile `3554:FA09`, release `2008`, `FF02:0002`, 20-byte Input/Output,
no Feature report, and the exact captured descriptor. One payload-free `0x4A` read is followed by a
bounded receive window (under three seconds, no resend). A valid reply must carry Report ID `13`,
command `4A`, one packet at index zero, exactly two payload bytes and a valid additive checksum.
The resulting keyboard has no lighting capability or settings writes. The separate opaque Feature
report is never used for battery data. Radio discovery is skipped only after a wired device has
successfully confirmed its identity and configuration. A failed wired candidate falls back to radio;
a confirmed USB board and its attached receiver still produce one keyboard tile.

Radio visibility is independent of battery availability after the exact receiver descriptor passes.
A timeout or malformed battery reply keeps the receiver tile, reports the error and omits an unknown
percentage. The existing battery refresh can recover on that same device without another descriptor
read or rediscovery. Foreign descriptors are still rejected before any output request.

2026-10-28 radio visibility regression: hardware-free tests cover timeout retention, malformed
telemetry, recovery, failed USB confirmation fallback and confirmed USB precedence. The rebuilt
release's read-only discovery found the owner's `3554:FA09` receiver, reported 76 percent on battery,
and recorded no errors, settings writes or firmware writes. A successful read at verification time
does not establish which transient condition caused the owner's earlier missing tile.

The core carries a generic power-source fact. Status `01` displays battery operation / not charging;
`10` displays external power with charging unconfirmed. Other values display unknown power status.
Diagnostics preserve the raw status and explicitly leave charging unknown. Displayed percentages
remain device estimates, with the connection-dependent discrepancy described above unresolved.

`tests/fixtures/by-tech-battery.json` contains the actual USB and radio reply bytes captured during
this investigation. `probe-by-tech-gpwb2401-radio.json` adds the sanitized receiver descriptor to the
cross-family admission suite. Hardware-free tests cover corruption, range, stale headers, timeout,
foreign mouse/headset identities and descriptors, one-request behavior, and wired preference.
The integrated release was run with `--lighting-probe by_tech`: it discovered one radio keyboard with
75 percent, battery power, no writable capability, no errors and no settings or firmware writes.

Integration validation: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`,
60 unit tests plus three admission tests, and the release build passed. Existing TCP and temporary
migration tests required execution outside the sandbox. The standard `--review` completed; device
headers at 720, 1040 and 1600 px were visually inspected with the long external-power status visible.

### Adaptive live battery updates (2026-10-28)

At the owner's explicit request, the earlier no-polling battery snapshot policy is superseded for
battery-capable devices. `core::Battery` refreshes an already identified device without discovery.
The real window requests one refresh every five seconds while not minimized, every five minutes
while minimized, and immediately on restoration. Failed reads back off to one minute when visible.
No background pool, permanent sleeping worker, idle loop, retained HID handle, or repeated descriptor
read is added. Only a deadline wake and a short-lived read worker run at each interval. Discovery and
lighting operations exclude concurrent telemetry. Failed reads clear the displayed stale percentage.

USB refresh rechecks the board password before the battery read, without reading lighting settings.
Radio refresh uses its already descriptor-validated endpoint and exactly one `0x4A` read. Neither
transport writes settings. The previously observed connection-dependent percentage variation remains
unresolved; these are device estimates rather than calibrated state-of-charge measurements.

The header now has a single battery row: proportional spring-animated fill in the existing battery
icon, a plug badge under external power, percentage and compact status. The UI interprets external
power below 100 percent as charging and 100 percent as charged, as requested by the owner. This is an
operational interpretation of the observed power state; a distinct hardware charging-current flag
has not been decoded. Unknown status does not imply charging. The discovery-time label was removed.

Live-telemetry validation: 63 unit tests and three cross-family admission tests passed, including
visibility deadline/restore/backoff behavior, exclusion of overlapping reads, stale-data removal,
and radio refresh without repeated descriptor validation. Fmt, clippy with warnings denied and the
release build passed. The final single-row indicator was visually checked with `--review` at
720, 1040 and 1600 px. No runtime CPU benchmark or multi-minute minimized-window hardware soak test
is claimed; the low wake rate is enforced by the tested scheduling policy.

### Per-device polling presets (2026-10-28)

The app now persists Automatic / Responsive / Eco telemetry modes in Settings. Automatic uses 5 s
for the open device card, 60 s elsewhere in the visible window, 5 min in background on battery and
1 min in background under external power below full charge. Each device owns its deadline, failure
backoff, last successful reading and timestamp. Only one read is dispatched at a time; completion
returns that device immediately rather than waiting for all devices. Failures do not alter another
unit's schedule. Hidden frames skip shell/model/page construction. Restore refreshes are staggered.
This supplies scheduling and freshness data for the planned tray battery display; no tray icons or
Windows notification feature is claimed. See `docs/ARCHITECTURE.md` for the complete preset table.

Validation of per-device presets: 65 unit tests and three admission tests passed, including legacy
settings migration/defaults, mode persistence, scope/power intervals, independent failure deadlines,
staggered restoration, one in-flight job, retained sample freshness and progression to another device
after a failed read. Fmt, clippy with warnings denied, release build and standard `--review` passed.
The settings selector was visually inspected at 720, 1040 and 1600 px. These checks do not constitute
a physical multi-device power benchmark or a native tray integration test.

### Owner observations, 2026-10-29

Owner describes Flash as a key-triggered bidirectional row pulse; Ripples originate at the pressed key. Rainbow cycles all hues together; Neon moves a dark gap left to right; Sine oscillates vertically without horizontal travel; Waterfall changes rows from top to bottom. Windmill resembles RainbowWheel. Fade, Scan and Storm appear unlit. Rain, Stars and Snake previews were about five times too fast. These observations guide previews, not measured firmware timing. No hardware write was performed for this change.

### Live lighting editing, owner request 2026-10-29

The owner explicitly requested immediate hardware application of mode, color, brightness and speed edits. The application now uses the existing backed-up configuration write/read-back path and coalesces edits with a 200 ms minimum dispatch interval. This is not a newly verified volatile command: persistence and memory endurance remain unchanged/unknown. No firmware, erase, reset, calibration or bootloader command is introduced. Speed byte 0 is preserved and means very slow; it is not Off. Hardware-free queue tests cover busy exchanges, final-value delivery and no retry after failure. Physical behavior of this UI path still needs owner comparison.


### Recovery preparation after vendor reset (2026-10-29)

The owner reported running the official driver's reset themselves. It changed illumination but did not restore keys; encoder rotation still works and Fn in radio mode lights R. A new read-only USB probe confirms the same identity. Lighting fields now parse (effect 11), while 53 configuration bytes differ from the earliest original backup, including unrelated fields. The standalone tool was rebuilt against this exact current block. Its fixed-packet test, clippy and build passed. Noninteractive execution was rejected by automatic approval review before launch. The revised tool requires the owner's direct Yes button press, then rechecks identity/current configuration before one original-config write and mandatory read-back. Physical recovery is pending.


### Owner-confirmed input recovery (2026-10-29)

After the owner explicitly requested backup recovery and pressed Yes in the native confirmation dialog, the standalone tool revalidated identity and the exact post-vendor-reset configuration. It preserved the damaged block, sent one 128-byte original-configuration write (command 0x04), waited 500 ms and read back the complete original block byte for byte. The owner then confirmed that the keyboard keys work again. No palette, key-matrix, firmware, bootloader or reset command was issued by this recovery. A later optional diagnostic did not obtain a wired identification and supplies no additional verification; the successful immediate read-back and owner's physical check are the recovery evidence.

Restoring the original configuration recovered key input; this establishes that the configuration state was relevant to the failure. It does not identify the precise write/framing/timing or firmware mechanism that damaged it. At that stage ordinary BY Tech writes and restore remained quarantined; recovery success alone was not used to enable them. Manual Apply is subsequently enabled by the explicit owner authorization documented at the top of this file. Original backups and local before/after recovery blocks are retained in ignored research storage. The subsequent reviewed explicit commit path and owner-controlled verification procedure are described at the top of this file.


### Reusable backup tooling and containment (2026-10-29)

ConfigurationBackup is now separate from Lighting in both applications. Wired BY Tech snapshots preserve the entire known 128-byte system block and 512-byte palette; this is not a full key-map/macro/display or firmware dump. The owner's exact tri-mode password/release and descriptor can recover only the system block, following the already owner-confirmed recovery procedure. Current blocks are saved before one write and full read-back follows after 500 ms. Damaged configuration no longer removes a correctly identified device from recovery discovery. Ordinary lighting writes are rejected and the palette packet builder refuses all writes. All read headers are checked in full.

AA55 separately captures and restores its known lighting block with its own identity, backup and verification checks; this new feature was not physically exercised on the co-author's keyboard. Radio and unsupported protocols expose no settings backup capability. Every archive remains bound to the original hashed endpoint/protocol. Reusable toolbar/standalone UI tests use synthetic devices; no new overnight hardware write is claimed. The earlier owner-confirmed single-block recovery remains the hardware evidence. Detailed incident analysis: INPUT-LOSS-2026-10-29.md.

Both applications now offer opt-in, rotating diagnostic logs without raw packets or private identifiers. Enabling logging is independent of enabling experimental device support and does not grant permission to write.
