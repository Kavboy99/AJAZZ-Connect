# Lusorium Studio 0.3.0

[Русская версия](README.ru.md)

Lusorium Studio (formerly the working name "AJAZZ Connect") is a universal application for configuring
and customizing PC devices from any manufacturer (keyboards, mice, lighting, fans, etc.); the current
version is a first slice with lighting drivers. Direction and rules: `AGENTS.md`, `docs/ARCHITECTURE.md`.

Windows x64: solid-color lighting through protocol drivers and passive USB/HID diagnostics; found devices
are shown as square tiles with a 3D model. The EXE needs no installation, Python or administrator rights.

A protocol is chosen by interfaces and confirmed by an exchange, never by a product name. The project is
developed on two keyboards of the same model but different platforms: the magnetic AK820 MAX Ultra
(SONiX, the co-author's) and the mechanical AK820 MAX `GPWB2401` (BY Tech `258A:010C`, the repository
owner's). The mechanical one now has the experimental `by_tech` driver, restored by static analysis of
the vendor package and not yet verified on the keyboard. Details: `docs/DEVICES.md`.

## Drivers

| Driver | Admission | Action | Rollback |
| --- | --- | --- | --- |
| SONiX AA55 | `0C45`, `FF68:0061`, Input/Output 65, a profile match, exact GET_INFO with matching VID/PID, known LED_EFFECT layout | Persistent color; read-back | LED_EFFECT saved to a backup file before the first write; button or file |
| SONiX Feature (experimental, off by default) | `0C45:8009/800A`, exact product name, `FF13:0001` Feature 65, `FF68` Output 4097 in the same USB container, exact START ACK | Persistent color; ACK check | None: the protocol cannot read the original effect |
| HID LampArray | Any VID/PID with `0059:0001`; report fields, order, widths and IDs are checked | Color of programmable lamps; any LampArray kind (keyboard, mouse, ...) | Autonomous mode by button or on close |
| OpenRGB SDK | Local SDK server on `127.0.0.1:6742`; Direct mode without Automatic Save; any device type | Color through the device's OpenRGB driver | Server state saved before the first write; button or file |
| BY Tech (experimental, off by default) | `FF00:0001` Feature 520 (Report ID 6), profile `258A:010C`, exact password reply `03 00 00 00 01 42` or `…01 5E` (single- or tri-mode board; USB only) | Persistent color: static effect in the 128-byte configuration block and the color table; read-back of both | Both blocks saved to a backup file before the first write; button or file |

Physical verification of these drivers is not recorded on either keyboard (`docs/DEVICES.md`, log). The
tests cover the codecs, the exchange order against a scripted bus, Microsoft's reference HID descriptor,
the published SONiX packets and the SDK against a test TCP server. No driver implements a bootloader,
erase or firmware load.

## Using it

1. Run the EXE. Devices are found by themselves: at startup and whenever one is plugged in or out
   (read-only; the operating system reports the change, nothing polls).
2. "Devices" shows them as tiles; "Refresh" forces a new search.
3. Open a device: its page shows one section per capability. In "Lighting" pick a color and click
   "Apply color". Badges say in advance whether the color persists and whether it can be rolled back.
4. OpenRGB devices appear when OpenRGB runs with its SDK server enabled; the application never starts it.
5. Experimental drivers are enabled in "Settings".

Backups: `%LOCALAPPDATA%\LusoriumStudio\backups`. Restoring from a file checks that the backup belongs to
the device. Do not run several programs that control the same device at once.

## Diagnostics

"Diagnostics" saves a JSON report with HID interfaces, their families (from `src/profiles/data`), report
descriptors and read-only identification replies. No serial numbers, device paths or key presses. The
application is for users: firmware and package analysis is not part of the interface. Developers have
`--inspect` (a vendor EXE/BIN is parsed as bytes and never run). For the mechanical `GPWB2401` firmware
work is not possible: the controller, memory map and a suitable image are not established.

Research notes (Russian): `docs/KEYBOARD-FAMILIES-RE.md`, `docs/ORIGINAL-UPDATER-RE.md`. Source copies
and hashes: `research/SOURCE-MANIFEST.json`.

## Building and the command line

Rust 2024. Structure: `docs/ARCHITECTURE.md`; adding a device or driver: `docs/ADDING-A-DRIVER.md`;
interface rules: `docs/DESIGN.md`.

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

Every command is read-only toward devices; results go to JSON or PNG files:

```powershell
lusorium-studio.exe --probe report.json
lusorium-studio.exe --inspect vendor.exe inspection.json
lusorium-studio.exe --lighting-probe sonix_aa55 aa55.json
lusorium-studio.exe --lighting-probe hid_lamparray lamparray.json
lusorium-studio.exe --lighting-probe openrgb openrgb.json
lusorium-studio.exe --review review-dir
```

`--lighting-probe` runs one driver's discovery; for `sonix_feature` that includes a START/FINISH pair, but
never SAVE or a color. `--review` renders interface states with synthetic data to PNG files (no device,
socket or window).
