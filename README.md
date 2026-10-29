# Lusorium Studio 0.3.0

[Русская версия](README.ru.md)

Lusorium Studio (formerly the working name "AJAZZ Connect") configures explicitly supported devices through its own proprietary device-control system and direct protocol adapters. Current development targets are the SONiX and BY Tech AK820 MAX hardware variants. Support is verified per hardware revision; no external RGB service is required. Direction and rules: `AGENTS.md`, `docs/ARCHITECTURE.md`.

Windows x64: driver-backed lighting and passive USB/HID diagnostics; found devices
are shown as square tiles with a 3D model. The EXE needs no installation, Python or administrator rights.

A protocol is chosen by interfaces and confirmed by an exchange, never by a product name. The project is
developed on two keyboards of the same model but different platforms: the magnetic AK820 MAX Ultra
(SONiX, the co-author's) and the mechanical AK820 MAX `GPWB2401` (BY Tech `258A:010C`, the repository
owner's). The mechanical one now has the experimental `by_tech` driver, restored by static analysis of
the vendor package; USB identification and USB/radio battery reads have been verified on the owner's keyboard. After the 2026-10-29 input-loss incident, original configuration recovery passed full read-back and the owner confirmed working input. The owner now authorizes manual lighting verification: Select/Apply are enabled for the identified tri-mode revision, with fresh reads, backup and separate palette/configuration read-back. Card selection and sliders never write automatically. Ordinary lighting restore remains unavailable; Settings provides the verified system-only recovery. Details: `docs/DEVICES.md`.

## Drivers

| Driver | Admission | Action | Rollback |
| --- | --- | --- | --- |
| SONiX AA55 | `0C45`, `FF68:0061`, Input/Output 65, a profile match, exact GET_INFO with matching VID/PID, known LED_EFFECT layout | Persistent color; read-back | LED_EFFECT saved to a backup file before the first write; button or file |
| SONiX Feature (experimental, off by default) | `0C45:8009/800A`, exact product name, `FF13:0001` Feature 65, `FF68` Output 4097 in the same USB container, exact START ACK | Persistent color; ACK check | None: the protocol cannot read the original effect |
| HID LampArray | Any VID/PID with `0059:0001`; report fields, order, widths and IDs are checked | Color of programmable lamps; any LampArray kind (keyboard, mouse, ...) | Autonomous mode by button or on close |
| BY Tech (experimental, off by default) | `FF00:0001` Feature 520 (Report ID 6), profile `258A:010C`, exact password reply `03 00 00 00 01 42` or `…01 5E` (single- or tri-mode board; USB only) | Read-only discovery/battery; explicit backed-up Select/Apply with per-block read-back on the identified tri-mode revision | System-only recovery in Settings; no automatic lighting restore |

Lighting-write verification of these drivers is not recorded on either keyboard (`docs/DEVICES.md`, log). The
tests cover the codecs, the exchange order against a scripted bus, Microsoft's reference HID descriptor,
and the published SONiX packets. No driver implements a bootloader,
erase or firmware load.

## Using it

1. Run the EXE. Devices are found by themselves: at startup and whenever one is plugged in or out
   (read-only; the operating system reports the change, nothing polls).
2. "Devices" shows them as tiles; "Refresh" forces a new search.
3. Open a device: the navigation collapses and the large model fills a fixed-height workspace, with
   identity and actions on the right. On wired BY Tech devices, Lighting opens a scrollable island
   with 18 native effects and Off. Settings offers brightness (0–4), speed where supported and an
   optional custom color. Saved device settings are loaded during discovery; choosing a preset keeps
   its saved palette until a custom color is enabled. The model is an illustrative preview.
   Press Apply to send the selected program: original blocks are backed up before the first write,
   only changed blocks are written, and both are read back. Restore and From backup remain available.
   Other drivers retain their explicit solid-color action and Solid/Wave/Reactive model previews.
   Encoder assignments remain preview placeholders. Scrolling and editing never write to hardware.
   Receiver-only connections and the offline driver catalog also show these native controls using
   illustrative values, with Apply disabled. For actual lighting control, connect the keyboard by USB
   and select its wired mode; a connected receiver alone exposes battery telemetry only.
   A confirmed receiver stays visible when its battery read fails; an unavailable percentage is not
   invented. Radio discovery also runs if an attached wired interface fails confirmation.
   Native effects retain framed cards and a hue wheel pinned above brightness/speed. Breathing,
   rain, ripples, stars, snake, scanning, windmill and the other programs have distinct illustrative
   previews. Firmware timings are not captured or emulated exactly.
   Native continuous modes now keep animating in the focused editor. Key-triggered previews remain
   dark until a key event; the owner confirmed hardware mode 7 behaves this way. Wired discovery
   reads saved speed, brightness and RGB slots, without treating a displayed palette as a color edit.
4. Experimental drivers are enabled in "Settings".

Backups: `%LOCALAPPDATA%\LusoriumStudio\backups`. Restoring from a file checks that the backup belongs to
the device. Do not run several programs that control the same device at once.

The default window is 1360 x 840 logical pixels (minimum 720 x 520). The settings inspector and device workspace stay in place without scrolling. Saturation updates the complete wheel; compact lighting controls fit the panel. Warnings sit below the model above the island. Back and Escape close settings, then the editor, then
return to the device list. The island keeps its own close button, which returns directly to overview.
Key highlights use input received by the focused application, with no global keyboard
hook, HID input reader, logging or saved key history. Modifier sides, Fn and OS-reserved keys are not
individually available through this input path. Wave plays a finite 1.6-second preview on selection;
native continuous effects preview automatically while focused. At rest the model texture is cached
and no animation requests another frame.
Models render on the GPU through wgpu/Direct3D 12, directly into the interface without copying frames
back to the CPU. Large temporary render targets are released when motion stops or the window hides.
The enlarged lighting/encoder view still follows the pointer, at 28% of the overview's rotation.
Camera, framing and panel transitions inside the device workspace settle without bounce; the restrained
opening overshoot is reserved for the transition from the device list.
Model framing interpolates toward the final layout fit, avoiding an intermediate enlargement when the
inspector and bottom island change the available width and height together.

Lighting colors use hue rings at full LED value: Solid/Reactive have one ring, Wave has three nested
rings and an independent color per ring. There is no black or dim-value selector. The center previews
the active ring. Arrow keys change its hue; Up/Down selects a ring. Mode names and the settings action
are inside each island card, with settings at its bottom trailing corner.
The rings use analytic edge coverage at native display density (at least 2x), including their inner edges.

The model retains its original 1536 px legend atlas, mipmapped anisotropic texture sampling and 4x MSAA.
Roughness-aware reflections, a shadow map and screen-space contact shading give the keycaps depth.
Light originates at the under-cap sockets and legend apertures, with local diffuse spill and restrained
HDR bloom from visible emission. Plastic albedo is preserved. This is a GPU lighting approximation;
the connected keyboard's LEDs and keycap transmission have not been measured.
The opened device uses a more directional studio rig, stronger contact shading, a subtle edge
vignette and a soft grounding shadow. LED gaps and legends carry HDR radiance with filtered bloom;
Flash and Ripples leave a finite, locally fading wake. The wake follows the model during camera
movement and expires within two seconds of the last key event, restoring the idle render cache.
Bound key surfaces use a separate 2048 px atlas with 192 px cells and outline-rasterized system-font
legends. Their plastic color is sampled from the source; baked highlights and distorted source letters
are replaced, rather than enlarged. Case textures remain original. Atlas construction happens once
with model loading; missing system fonts retain the original asset.

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
lusorium-studio.exe --review review-dir
lusorium-studio.exe --review review-dir studio-palette
lusorium-studio.exe --review-window gpu-metrics.json
```

`--lighting-probe` runs one driver's discovery; for `sonix_feature` that includes a START/FINISH pair, but
never SAVE or a color. `--review` renders interface states with synthetic data to PNG files (no device,
socket or window).
`--review-window` opens a synthetic device in the native GPU interface, records rendering counters and
UI update times, then closes after 15 seconds. It does not discover devices or open HID handles.
GPU requirements and measured idle costs: `docs/GPU-PERFORMANCE.md`.

Developer previews: Settings > Developer mode reveals Show all supported devices. The offline catalog uses built-in profiles for compiled drivers and opens model/passport previews without creating device sessions. Models keep full resolution and 4x MSAA during interaction.


Catalog tiles group explicitly linked model variants, show a multiple-driver caption, and offer an offline driver selector inside the model page. Connected devices show their actual driver in the live list; the separate catalog still shows each model once with its available driver variants. A shared 3D asset does not imply identical electronics. The generic mouse in --review is synthetic. AJ159 is an offline placeholder with a supplied mouse-and-dock 3D model; no driver, detection, DPI, button or lighting control is implemented. Find it under Settings > Developer mode > Show all supported devices.




Device pages open by expanding the selected tile into the content surface. The same interactive 3D stage fills the workspace, beside the device name, driver, connection state, battery snapshot and actions. Back, Escape and the Devices navigation item reverse the reveal. BY Tech battery updates use a saved Automatic / Responsive / Eco preset in Settings. Automatic refreshes the open device card every 5 seconds, other visible devices every minute, and background devices every 5 minutes on battery or every minute during observed charging; restoration refreshes immediately with staggered reads. A single row shows proportional battery fill, percentage and charging status derived from the observed power state. Failures back off independently per device; there are no permanent worker threads or repeated device searches. Enable experimental drivers in Settings. Reported percentages can vary by connection/power state (see `docs/DEVICES.md`).

The device return now restores list brightness continuously and reveals tile labels before handoff. Opening gives only the model a restrained spring overshoot (about 0.6%); the surface stays steady. Both motions retain velocity on reversal and stop repainting after settling.

The keyboard preview uses flat keycap tops with the row slope preserved. The reference status decal (lettering and baked shadow beside Enter) is omitted; legends are rasterized at their final horizontal scale to preserve clean strokes on wide keys.

Lighting previews follow the owner's observed modes: key-triggered row flashes and rings, uniform rainbow cycling, row-based waterfall, and dark Fade/Scan/Storm. App lighting preferences take priority over device colors and are stored locally. Saturation includes white and pastels. Persistent lighting edits require explicit Apply. BY Tech manual lighting writes are enabled for owner testing on the identified tri-mode revision. Card clicks and sliders edit the local preview; Select beside Settings or Apply in the inspector commits the draft with a backup and read-back. Select remains visible when a card is only previewed and is hidden only on the last device-confirmed mode; the inspector can apply edits to the active mode. Battery reads preserve button appearance and accept a queued explicit click. Commit progress keeps the button width fixed and fades between the checkmark icon and spinner. The checkmark stays on the last device-confirmed mode until a commit succeeds. Timing is approximate.

The workspace keeps routine editing free of standing information banners. Experimental status appears once as small, light orange text below Back. USB identifiers, firmware and release codes appear on hover over the connection status; battery remains visible separately. The preview-without-writing caption and the separate technical details section are removed. Only a current connection blocker or error uses the message area; loading stays on the button and the active mode on its card.

The caption and navigation pane share native DWM Acrylic and follow the application's light/dark theme together. The full-window alpha region stays enabled so native repaints and minimize/restore do not replace the navigation material with an opaque bitmap. Windows controls material opacity and the inactive fallback; the content area stays opaque.

Ordinary startup also clears the native HWND client bitmap to zero after the first presentation and keeps a black class brush for resize regions. This prevents the white strip that previously covered expanded navigation despite the DWM Acrylic attributes being enabled. Packaged EXE startup is checked separately from the native review window.

For a hardware-free native material review, run `target\release\lusorium-studio.exe --review-window verification\native-gpu.json dark` (or `light`). It closes after 15 seconds, saves GPU metrics, and never saves review preferences. Unlike offscreen `--review`, this window includes the system-composed caption and Acrylic.

Slider handles are circular. Speed 0 is the slowest moving setting, never an animation-off state. Live lighting queues survive page changes; a failed exchange cancels pending changes without automatic retry. Opening a page or reconnecting alone never sends lighting writes.


## Backups and recovery

The standalone lusorium-backup.exe is placed in the project folder. The main application exposes Save backup / Restore from backup in Settings > Backups, keeping the device workspace focused on editing. Both use explicit file selection and a final restore confirmation; capture never changes settings. Read BACKUP-README.ru.md for the Russian step-by-step guide and docs/INPUT-LOSS-2026-10-29.md for the incident evidence.

Archives (.lsbackup) contain size-bounded GZip JSON with a payload SHA256; existing driver JSON backups remain importable. Every recovery validates the target identity/protocol, preserves current state before writing, and checks read-back without an automatic retry. The original pre-incident archive is retained locally in device-backups/original-before-incident.lsbackup.

Coverage is explicit: wired BY Tech saves the 128-byte system configuration and 512-byte palette; the owner-verified tri-mode revision can restore the system block only. Explicit lighting Select/Apply uses separately verified palette/configuration writes, while recovery still restores only the system block. AA55 saves/restores its known lighting block. Unsupported devices and radio settings have no backup/recovery capability. These are not universal full-memory dumps; macros, key mappings, display contents and firmware are excluded. No new overnight hardware writes were used to validate the feature.


Developer settings also provide one persisted, opt-in logging switch for all drivers in both applications, plus Open logs folder. Typed rotating JSONL traces include action/operation order, command codes, sizes, durations, sanitized error categories and Rust panic source locations. No raw packets, key presses, serials or device/file paths are logged. Files roll at 2 MiB with four recent session files retained; disabled logging has no worker or idle IO. See docs/ARCHITECTURE.md.


Backup UI update: use Settings > Backups, the first card, then Find devices. This workflow is independent of the ordinary experimental-driver setting. The standalone utility is now a separate compact application with no Studio shell or 3D viewer; extract lusorium-backup-portable.zip and run its EXE. Unknown HID keyboard interfaces are visible but receive no protocol commands and cannot be backed up/restored until their protocol is supported. Full backups of every setting on arbitrary keyboards remain unsupported.

Lighting correction: Brightness and Speed now use the owner-confirmed BY Tech field order; the current program card omits Select. Background battery reads no longer flash Apply, and an explicit click waits for telemetry to finish. Saturation uses direct RGB-channel HSV without extra gamma whitening; actual LED color response still requires owner comparison.

Device capability actions use medium square tiles with rounded corners, centered icons and labels, laid out in a responsive grid. Lighting, encoder and future actions share the same component; the pattern is defined in docs/DESIGN.md section 5.2.
