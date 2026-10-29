# Lighting field order and color audit, 2026-10-29

This audit concerns the mechanical AK820 MAX GPWB2401, BY Tech `258A:010C`, Feature Report 6,
520 bytes, tri-mode profile password/release `03000000015e-0100`. It does not establish behavior
for the magnetic SONiX keyboard, AK820 Pro, or another keyboard sharing the model name or VID/PID.
No vendor program was executed and no HID request or device write was made for this audit.
The work inspected downloaded files, source code, and existing automatic pre-write snapshots.
Application behavior was not changed during the research.

## Findings and confidence

| Question | Finding | Evidence and limit |
| --- | --- | --- |
| Were brightness and speed reversed in our former adapter? | Yes; very high confidence | Fresh vendor binary, exact profile-to-packet trace, independent implementations, and the owner's physical observation agree. |
| Correct format? | `[brightness, (speed << 4) | color_slot]` at `0x38 + 2 * effect` | This is the configuration payload offset; add the eight-byte header for a complete HID report. |
| Are there only seven possible colors? | No such restriction in the host protocol | Seven stored RGB slots per effect include a replaceable custom slot; they are not a seven-color output-depth specification. |
| Can intermediate RGB values be stored? | Yes, observed in existing snapshots | A fixed sample of 30 snapshots from one endpoint contains 29 active colors with intermediate channel values. This proves storage, not optical output. |
| Are 256 optically distinct levels per channel established? | No | The LED part, PWM resolution, firmware transfer function and measured channel response are not established. |
| Does changing the HSV conversion guarantee accurate pastel light? | No | The numerical difference is proved; the physical improvement still needs an owner comparison or measurement. |
| Should saturation be removed and white replace a hue? | Not justified by the evidence | Preserve intermediate RGB selection unless controlled hardware tests demonstrate an actual restriction. |

## Fresh package and provenance

The [AJAZZ download page](https://ajazz.net/pages/ajazz-drivers) separates the ordinary AK820 Max
package from the HE packages. Its ordinary Max link was followed again on the audit date:

- [Published ZIP](https://cdn.shopify.com/s/files/1/0554/6678/6869/files/AJAZZ_AK820_MAX_driver_V1.0.zip?v=1758597454)
- ZIP SHA256: `a264ecb265b3c1eb3d60b6138e89b785f39f83c3b668f564be22107e30b0d6cf`
- Embedded RAR SHA256: `b42fd409b251402eb43d0c07385093301fda6aedacffbb8d67493f89ae590274`
- Extracted `OemDrv.exe` SHA256: `fc7109b0467602a7624b2a83402bae12255ad5ad2e9483a2c4750c584231c06c`
- Tri-mode `KB.ini` SHA256: `e8ae2220482247ed1348deb33832cbcfe67628595befa47b791a44ad381690dc`

The newly downloaded RAR is byte-for-byte identical to the preserved source archive. Extracting its
installer using 7-Zip and innoextract reproduces the same `OemDrv.exe` hash. The installer and
extracted vendor EXE/DLL files were never launched or loaded. Their presence in a download channel
does not by itself establish a vendor signature or a firmware hardware match; the exact profile
and previously identified endpoint provide the relevant local match.

All vendor material stays in ignored `research/downloads/ak820max-color-audit/`. The package contains
no `.bin`, `.hex` or `.fw` image from which this keyboard's LED output routine could be established.

## Exact brightness/speed evidence

The matching `Dev/kb/Thr/KB.ini` declares `Fw=24`, `VID=0x258a`, `PID=0x010c`, the known password,
`LightHW=0,1,2,3,4`, `SpeedHW=0,1,2,3,4`, and `RGBIndex=0,1,2`.

The complete chain in the freshly extracted binary is:

| Stage | Virtual address | Observed operation |
| --- | --- | --- |
| Speed profile key | `0x410CE6` | References the UTF-16 `SpeedHW` string at `0x5FCD54`. |
| Speed lookup destination | `0x410D0B` | Loads object offset `0x2E50` for the parsed array. |
| Brightness profile key | `0x410E00` | References `LightHW` at `0x5FCD94`. |
| Brightness lookup destination | `0x410E29` | Loads object offset `0x2E70`. |
| Pair destination | `0x48CE81` | EDI starts at configuration `+0x3B`, the second byte for effect 1. |
| Brightness value | `0x48CE9C` | Loads DL from the `0x2E70` table. |
| Speed value | `0x48CEA3` | Loads CL from the `0x2E50` table. |
| Packing | `0x48CEB7..0x48CEBE` | Masks the color slot to four bits, shifts CL by four, combines and stores the second byte. |
| First byte | `0x48CEC4` | Stores brightness DL at `[EDI-1]`. |
| Next effect | `0x48CECF` | Advances EDI by two bytes. |

For example, effect 7 uses payload bytes `0x46` and `0x47`. `[03,17]` means brightness 3,
speed 1, color slot 7. Changing brightness to 4 changes only `0x46` to `04`; changing speed to 2
changes only the high nibble at `0x47`, producing `27`. This agrees with the independent regression
test in `src/drivers/by_tech/tests/lighting.rs`. The owner's report that our former Brightness
control changed speed is a separate hardware observation supporting this interpretation.

The offline [audit tool](../tools/audit_by_tech_lighting.py) independently parses the PE using the
Python standard library and checks the exact source strings and 19 instruction-byte anchors.
It refuses an unrecognized binary hash. It does not emulate or execute the vendor binary.

```powershell
python tools/audit_by_tech_lighting.py `
  research/downloads/ak820max-color-audit/unpacked/app/OemDrv.exe `
  research/downloads/ak820max-color-audit/unpacked/app/Dev/kb/Thr/KB.ini `
  --output research/downloads/ak820max-color-audit/vendor-audit.json
```

This audit passed. A Capstone disassembly of the same ranges was saved separately in the ignored
audit directory. Corrected application code reads and writes the same layout in `state.rs`,
uses it for native metadata, and reports it in passive diagnostics. Tests cannot alone establish
firmware behavior, but here they are checked against a vendor-derived byte map rather than serving
as the map's only source.

## Independent public implementations

[DeetsRGB's protocol notes](https://github.com/deets-137/DeetsRGB/blob/main/docs/protocol.md)
describe brightness first, speed in the next high nibble, and user RGB in palette slot 0.
[AULA F87 protocol notes](https://github.com/veysiemrah/aula-rgb-controller/blob/main/tools/protocol_notes.md)
document the same pair ordering from USB captures. Their complete-report offset includes the header.
These are corroborating observations from their authors, not tests on this owner's keyboard.

They also document differences: DeetsRGB reports a different brightness range and gapped effect
layout; the AULA profile uses another password. Some published observations disagree about the
direction of increasing speed. No foreign command, range, effect table, MCU identity or streaming
assumption was transferred into our adapter. The direction and physical timing of this firmware's
speed steps remain separate from identifying which field controls speed.

## Color: stored values versus emitted light

At `0x48D780..0x48D7B4` the vendor writer loads a color DWORD and copies its low three bytes into
the output using `RGBIndex`. For the matching profile that is R, G, B. This path does not reduce
channels to 0/255 or choose a nearest fixed color. The configuration selects a palette slot;
slot 0 can hold arbitrary RGB bytes. Modes whose profile has no custom-color flag are a separate
restriction: the availability of a color picker cannot make a firmware-owned rainbow mode accept
a custom single color.

A fixed, anonymized inspection of 30 existing automatic pre-write snapshots found one device
identity and the expected password/release. Twenty-nine active colors contained intermediate
channel values. Examples include `211,240,255`, `247,255,211`, and `23,177,254`.
Only counts, protocol data and representative color triples were retained in the audit summary;
endpoint hashes and source filenames were omitted. These snapshots came from fresh device block
reads before previous owner actions. No new read or write was performed to collect them.

This rules out a *storage/interface* that only holds a small fixed palette. It does not rule out
firmware quantization, channel gain differences, low-resolution PWM, mode-dependent processing,
or optical effects after the values are stored. The exact LED package and PWM driver remain unknown.
Marketing descriptions of another AK820 MAX revision cannot fill this gap.

## Saturation and the former gamma conversion

Our former path converted egui's linear `Hsva` into `Color32`, which applies sRGB encoding.
The present picker instead defines HSV directly over the numeric 0..255 RGB channels and sends
those same bytes. For cyan at full value:

| Saturation | Present RGB-channel HSV | Former gamma-encoded result |
| --- | --- | --- |
| 100% | `0,255,255` | `0,255,255` |
| 75% | `64,255,255` | `137,255,255` |
| 50% | `128,255,255` | `188,255,255` |
| 25% | `191,255,255` | `225,255,255` |
| 0% | `255,255,255` | `255,255,255` |

Thus the former mapping numerically raised the weaker channel and moved the triplet toward white.
Reducing HSV saturation at full value also intentionally moves the result toward white.
There is no saturation field on this wire protocol: the application computes RGB first.
HLS, HSV, linear-light and sRGB percentages are not interchangeable measurements.

The numerical explanation is verified. It would be too strong to claim that removing this encoding
alone proves the physically correct transfer curve or explains all observed whitening. Gamma
correction is not universally wrong for LEDs: [Analog Devices explains its role in perceived LED
brightness](https://www.analog.com/en/resources/design-notes/using-lookup-tables-to-perform-gamma-correction-on-leds.html).
[ams OSRAM AN117](https://look.ams-osram.com/m/254283943517a1e/original/Ambient-lighting-design-utilizing-RGB-LEDs.pdf)
describes independent RGB emitters and the measurements/calibration needed to hit a target color.
Neither document identifies the components or transfer function of this keyboard.

## What would close the remaining uncertainty

An owner-controlled comparison should hold the Static effect and brightness constant, vary only
saturation through several steps, and use the existing explicit Apply and read-back path. Compare
the same RGB triples in the vendor interface if exact RGB entry is available; comparing percentage
labels from different color models is not sufficient. Do not automatically cycle persistent writes.

Fixed exposure and white balance can support a qualitative comparison; an ordinary photograph or
subjective observation cannot establish 8-bit PWM depth or a colorimetric transfer matrix. Exact
claims require optical measurement, electrical PWM observation, or independently matched firmware
analysis. None was performed here. No ISP, reset, calibration, firmware dump or bootloader operation
is proposed or needed to support the confirmed field-order correction.

For now, the evidence supports keeping continuous RGB/saturation selection. It does not support
replacing it with a few saturated colors and white as a supposed hardware necessity.
