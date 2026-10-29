# Device workspace contrast and light persistence

The opened device previously used the inventory light rig. Its broad fill and reflections competed
with LED emission, while reactive waves lost most of their tail through sRGB intensity scaling.

## Changes

- Workspace pose carries a studio blend, included in the render cache key. It follows the existing
  reveal spring, reduces fill/environment contribution and changes the key-light direction.
- Depth contact shading scales with render size. A subtle model-edge vignette and a theme-token
  stage shadow add depth without covering the identity, actions or settings inspector.
- Only bound sockets and legend apertures emit. Their HDR radiance, local reflected spill and bloom
  are stronger; the plastic albedo and full-resolution legend atlas remain independent.
- Bloom first integrates thin light sources over a 4x4 footprint, then applies symmetric Gaussian
  blur at quarter resolution. The two existing temporary bloom targets are reused. This adds one
  quarter-resolution pass, with no new large texture or frame-history buffer.
- Flash and Ripples have a bright moving crest and an exponential wake. Fade and Reactive retain
  local decay. Persistence multiplies linear light energy, avoiding gamma-crushed tails. All key
  events expire within two seconds. There are no ghost images when the camera moves.

The changes affect preview rendering only. They do not alter device protocols, persistence,
admission, selected hardware colors or write actions. The scene is an illustration, not a measured
LED simulation.

## Review

`--review DIRECTORY studio-depth-` adds matched overview, Solid, Flash and minimum-height Ripples
scenes in both themes at 720, 1040 and 1600 px. Key events are synthetic and the review never opens
hardware. Frames 65 and 78 show the crest and wake; final frames verify idle GPU counters, released
work attachments and no repaint. Existing editor/inspector, pointer and reversal scenes also apply.

Before images: `verification/lighting-depth-before/` (Ripples) and
`verification/lighting-depth-before-overview/` (overview). Final results and measured limits are
recorded in `STATUS.json` under `workspace_lighting_depth`.

The final release passed fmt, clippy with warnings denied, 115 unit tests and 3 admission tests.
The full test suite needs unrestricted temporary-directory renames and local test TCP sockets.
Twenty-one review scenes passed at three widths (63 final states), including both themes,
minimum-height inspectors, close-up legends, restrained zoomed pointer tracking and reveal reversal.

The matched wide Ripples opening sample changed from 2.205 ms mean to 3.102 ms mean / 4.751 ms max.
These offline CPU frame timings include GPU synchronization and readback; they are not native FPS.
The final native synthetic window on RTX 3080 performed zero readbacks and released work targets.
Its UI updates averaged 0.267 ms (10.530 ms max), excluding GPU execution/presentation. Pointer input
resumed after first idle, so its five-second process sample (500 ms CPU, 219.45 MB working set,
602.54 MB private memory) describes active interaction and cannot establish idle CPU. Static and
expired-reactive offline scenes independently verified unchanged render counters, zero work targets
and no requested repaint after settling. No physical keyboard was exercised.
