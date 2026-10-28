# GPU workspace review (2026-10-28)

Material follow-up: bound key surfaces now replace their baked photo atlas with independent 192 px
cells in a 2048 px atlas. System-font outlines provide readable legends and matching emission masks;
sampled plastic color remains separate from studio lighting. The source case texture is retained.
The atlas is generated once during loading; absent system fonts retain the source surfaces. Exposure
and key/fill intensity were reduced to preserve light plastic detail; the editing camera exposes more
sidewall depth. Measurements below predate this additional atlas and are historical baselines.
The material revision passed all 76 tests, fmt, clippy with warnings denied, and all 31 offline scenes
at 720/1040/1600 px (`verification/materials-final/complete.json`). The release executable is
`verification/lusorium-studio-materials.exe`; the existing native executable was open and locked during
the build, so it was preserved. No connected hardware was used.

The owner requested GPU acceleration, lower active CPU cost, smooth hue rings, pointer tracking
after zoom and deeper lighting. The native backend now uses wgpu 27 over Direct3D 12 for both egui
and the device stage. It requires a compatible hardware GPU/driver; the offline review rejects CPU
adapters. No transport, device admission or hardware write behavior changed in this revision.

## Rendering and lifetime

- `ui/viewer/gpu/mesh.rs` uploads each visual's geometry, native-resolution material atlas, mipmaps,
  legend mask and precomputed local emitter weights once. Materials use anisotropic sampling.
- `scene.wgsl` provides roughness/metallic GGX shading, studio fill/reflections, a filtered shadow map,
  per-key depression, legend/socket emission and local spill. `post.wgsl` adds depth-based contact
  shading, bounded quarter-resolution bloom and tone mapping. The scene uses 4x MSAA. This is an
  approximation of backlighting; LED output and keycap transmission have not been measured.
- A changed pose, effect or size renders into a cached output texture. Native egui samples that
  texture directly without copying pixels back to the CPU. Capacity grows in 128 px buckets rather
  than reallocating on every animated size change.
- HDR, multisample, depth, shadow and bloom attachments exist only during rendering. They are dropped
  at rest or when hidden; static mesh/material data and the resolved image stay cached. There is no
  animation thread, render timer or permanent GPU submission loop. Wave demonstrations remain finite.
- Zoomed lighting/encoder views keep spring-based pointer tracking with 28% of the overview's range.
  Hue ring coverage is calculated analytically at native display density with a 2x minimum. Its
  cached spectrum does not rebuild when only the marker moves.

The host uses safe Rust and wgpu validation; no application `unsafe` was added. Shader syntax and
semantics are checked with naga in a hardware-free unit test.

## Native synthetic window

Release build, Windows, NVIDIA GeForce RTX 3080. `--review-window METRICS.json` opens one synthetic
keyboard in the normal native GPU renderer without device discovery, HID handles or telemetry. It
records counters and UI update durations, then closes after 15 seconds. A separate process sample
after startup measured the same process over a five-second idle interval.

| Measurement | Result |
| --- | --- |
| Process CPU during five seconds of rest | 0 ms at the process counter's resolution |
| Working set | 182.83 MB |
| Private memory | 257.21 MB |
| Mesh uploads | 1 |
| Frame readbacks | 0 |
| Model renders at first idle / exit | 51 / 51 |
| Retained working views at idle / exit | 0 / 0 |
| Native UI update mean / maximum | 0.288 / 2.015 ms |

Evidence: `verification/gpu-idle-small-native.json` and `verification/gpu-idle-small-process.json`.
The default wgpu allocation hint previously yielded a 232.98 MB working set in this test. Switching
to `MemoryHints::MemoryUsage` reduced it to 182.83 MB. This is one sample per configuration, not a
cross-device memory guarantee; the earlier 120 MB guideline is still exceeded. Driver/runtime memory
is included. UI update durations exclude GPU execution and presentation and do not establish frame
rate. GPU utilization was not measured; counters establish that the model submitted no new work
after settling. This synthetic test does not measure background battery telemetry.

## Offline suite

`--review verification/gpu-checked` passed 31 scenes at 720, 1040 and 1600 logical pixels: 93 final
states plus intermediate transitions. It covers island close, settings, key highlights, narrow and
minimum-height layout, nested hue rings, reversals, pointer tracking after lighting/encoder zoom,
and no further repaint/model submissions at rest. The zoomed lighting trace settled at approximately
-0.037 radians yaw when the pointer entered the stage, confirming subdued tracking instead of a
frozen camera. All settled GPU scenes released their work attachments.

Offline screenshots use the same GPU model passes, then copy model images into the existing CPU
interface compositor. Their timings include GPU synchronization/readback and exclude PNG export,
presentation and OS compositing. At 1600 px, settings averaged 4.421 ms (max 7.220), Wave 5.050 ms
(max 9.000), and encoder zoom 9.015 ms (max 36.507). They are regression evidence, not native FPS.
The native path above has zero frame readbacks. No interactive desktop screenshot was captured:
computer-use access timed out, so visual review used the offline images.

`cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, all 76 tests (73 unit and
3 admission tests), and `cargo build --release` passed. No connected device was used for this review.
