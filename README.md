# AJAZZ Connect 0.2.0 (working name)

[Русская версия](README.ru.md)

The name is a working name and will be replaced. The project is moving toward a universal application for
configuring and customizing PC devices from any manufacturer (keyboards, mice, lighting, fans, etc.); the
current version is a first slice for keyboards. Direction and rules: `AGENTS.md`,
`docs/ARCHITECTURE.md`.

Windows x64: stock-protocol color setup for various keyboards, USB diagnostics and static analysis of
firmware EXE/BIN files. The finished EXE needs no installation, Python or administrator rights.

The program does not tie its settings to a single firmware 1.36, to magnetic switches or to the Max Ultra
name. The protocol is chosen by interfaces and confirmed by an exchange. The project is developed on two
keyboards of the same model but different platforms: the magnetic AK820 MAX Ultra (SONiX, the
co-author's) and the mechanical AK820 MAX `GPWB2401` (BY Tech `258A:010C`, the repository owner's).
Color control currently works only for the SONiX branch; the mechanical one is recognized by diagnostics
but has no adapter yet. Details: `docs/DEVICES.md`. Development rules: `AGENTS.md`.

## Control methods

| Adapter | Admission criteria | Action | Recovery |
| --- | --- | --- | --- |
| AJAZZ AA55 | SONiX AK820, FF68:0061, Input/Output 65 with Windows Report ID, exact GET_INFO with matching VID/PID, known LED_EFFECT format | Stock persistent color, saved setting; read-back | The original LED_EFFECT block is saved before the first write; button or JSON |
| SONiX Feature, experimental | 0C45:8009/800A, AK820/AK820 MAX, FF13:0001 Feature 65, FF68 Output 4097 of the same USB container, exact START ACK | Persistent color, saved setting; ACK check | The original effect cannot be read; no automatic rollback, the mode can be chosen with Fn on the keyboard |
| HID LampArray | Any VID/PID, standard keyboard 0059:0001; report fields, order, widths and IDs are checked | Color of programmable lamps without replacing firmware | Autonomous mode by button or on normal close |
| OpenRGB SDK | Local SDK Server; the keyboard is already supported by OpenRGB itself; Direct without the Automatic Save flag | Persistent color through the existing OpenRGB driver | Server state is saved before the first write; button or JSON |

Physical verification of the new adapters is not recorded on either of the two keyboards
(`docs/DEVICES.md`, log); no adapter applies to the mechanical GPWB2401. The tests cover the codecs,
Microsoft's reference HID descriptor, the published SONiX packets and the SDK against a separate test TCP
server. An ACK or server-state check does not replace observing the color on the keyboard. No adapter
implements entering the bootloader, erasing or loading firmware.

## Running

1. Unpack `AJAZZ-Connect-0.2.0.zip` and run `AJAZZ-Connect.exe`.
2. For AJAZZ, connect the cable, choose USB, and close other configuration programs.
3. On the "Lighting" tab choose an adapter and click "Find keyboards".
4. Choose a keyboard and a color and click "Apply color".
5. For OpenRGB, first enable the SDK Server on port 6742. The application does not start that server and
   a mere SDK connection does not extend the list of devices it supports.

AA55/OpenRGB backups: `%LOCALAPPDATA%\AjazzConnect\backups`. Restoring from a file checks that the backup
belongs to the device and has a valid format. SONiX Feature has no command to read the original effect;
the application shows this limitation in advance. Do not run several programs that control the same
keyboard at once.

## GPWB2401 and firmware

The "Diagnostics and firmware" tab saves a JSON with USB interfaces, protocol families and the available
GET_INFO. Choosing an EXE/BIN extracts the active image and metadata without running the EXE. A ZIP/RAR or
an unknown format must not be renamed to BIN. For the mechanical GPWB2401 (`258A:010C` over USB, BY Tech)
firmware is unavailable: the controller, memory map and a suitable image are not established. The name of
an archive such as AK820MAX proves neither the switch type nor the universality of an image.

Detailed analysis (Russian): `docs/KEYBOARD-FAMILIES-RE.md`; analysis of the original flasher (Russian):
`docs/ORIGINAL-UPDATER-RE.md`. Local copies of sources and their hashes are listed in
`research/SOURCE-MANIFEST.json`.

In the locally extracted AK820MAX V1.0, BYCOMBO4 and USB 258A:010C, RF 3554:FA09 are confirmed. This is
the branch of the mechanical GPWB2401 (the wired ID was confirmed by diagnostics on 2026-10-28). Its
configs are described in research/AK820MAX-V1-BYTECH.json; SONiX commands are not sent to it, and native
BY Tech writing is not enabled yet.

## Sources and checks

Rust 2024. `src/aa55.rs`, `src/sonix_feature.rs`, `src/lamparray.rs` and `src/openrgb.rs` split the
transports. `src/families.rs` holds passive classification, not permissions to write firmware. `tools/`
contains a standalone SONiX ISP model and a static analyzer: they do not open HID. Part of the extended
analysis requires pefile/capstone/Unicorn/Ghidra; the finished EXE does not depend on them.

```powershell
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
```

`build.ps1` uses local GNU tools if available; otherwise plain Cargo. The CLI performs diagnostics and
analysis only:

```powershell
AJAZZ-Connect.exe --probe report.json
AJAZZ-Connect.exe --inspect vendor.exe inspection.json
AJAZZ-Connect.exe --lighting-probe aa55 aa55.json
AJAZZ-Connect.exe --lighting-probe feature feature.json
AJAZZ-Connect.exe --lighting-probe lamparray lamparray.json
AJAZZ-Connect.exe --lighting-probe openrgb openrgb.json
```

During its check `feature` opens and closes a START/FINISH session; SAVE and color changes are not sent.
The `settings_writes=false` fields of the diagnostic report refer to this check, not to the user's
actions with the "Apply color" button.
