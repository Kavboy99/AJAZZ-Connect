# Devices and compatibility

A registry of the devices, from any manufacturer and of any kind, on which development is done, and of
everything known about their protocols. For now it holds two keyboards; new devices (mice, fans,
lighting) are added the same way using the scheme in section 4. Update this file after every new
verification. Statuses: `identified` → `read` → `written by owner` → `confirmed`. Anything not verified
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
| Status | Identified from project materials. `--probe` has not been run on this PC | **Identified by `--probe` on 2026-10-28** |
| Manufacturer driver | `AJAZZ_AK820_MAX_Single_Mode_RGB_Magnetic_Axis_Keyboard_Driver_Installer` V2.06.01 | `AJAZZ_AK820_MAX_driver_V1.0` (BYCOMBO4) |
| What the application can do | Color via AA55 (main), SONiX Feature (experimental), diagnostics | Diagnostics and classification only |
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

Full hashes are in the fixture. The `FF00:0001` channel with a 520-byte Feature report matches, by size,
the description of the Sinowealth `010C` controller in OpenRGB (Report ID 6). **That is a size match, not
a confirmation of the command format for this model.** Until it is verified by reading, commands from
other projects must not be copied.

The `3554:FA09` radio receiver: vendor interfaces `FF02:0002` (20 / 20 / 0, SHA256 `63c9063b…9c0406`)
and `FF04:0002` (feature 8, SHA256 `3c311f5d…a2cbb8`). It shows only the radio channel and says nothing
about the keyboard's board. The wired protocol does not apply to the radio.

The owner's PC also has another receiver, `3554:F52D`: by all signs it is a mouse receiver, unrelated to
the keyboard, and ignored in reports.

**What is unknown** (must be established before writing a color driver):
- The lighting packet format on Report ID 6 for this model and its length.
- Whether the color persists in keyboard memory, and how to read the current state (needed for rollback).
- Whether the commands are the same in wired mode and in 2.4G/Bluetooth (a settings block must not be
  carried between modes without verification: third-party projects report input failures after such a
  transfer).
- The controller and bootloader (for firmware: not before the items above, and only with the owner's
  separate consent).

Plan: static analysis of `OemDrv.exe` from V1.0 (it writes via `HidD_SetFeature`) without running it →
reading state with a Feature report → only then writing a persistent color on a button press.

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
| OpenRGB SDK | If supported by OpenRGB itself | If supported by OpenRGB | Local server at `127.0.0.1:6742` |
| BY Tech | No | **Target**, not implemented | `258A:010C` + a confirmed format |

Every "No" in the GPWB2401 column was checked against the code and the probe of 2026-10-28; fixture
tests should lock this in (`docs/ADDING-A-DRIVER.md`, rule 3).

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

## 4. How to add a device or a verification result

1. `ak820max-assistant.exe --probe report.json` (read-only).
2. Make sure the JSON contains no serial number or Windows paths; put the fixture into `tests/fixtures/`.
3. Add a section to this file: identifiers, interfaces, protocol source, what is unknown.
4. Add a row to the log below. For a write to a device: who did it and on whose hardware.

## 5. Verification log

| Date | What | Device | Who | Result |
| --- | --- | --- | --- | --- |
| 2026-10-28 | `--probe` over 2.4G (receiver) | GPWB2401 | owner | Only `3554:FA09` is visible; the board is not |
| 2026-10-28 | `--probe` over USB | GPWB2401 | owner | `258A:010C`; no SONiX/AA55/LampArray; read-only, no writes |
| 2026-10-27 | Static check of the Drv packages | n/a | audit | See `docs/DRV-COMPATIBILITY-AUDIT.md` |
| n/a | `--probe` over USB | Magnetic Ultra | co-author | Not done |
| n/a | AA55 color write | Magnetic Ultra | co-author | Not recorded in the repository |
