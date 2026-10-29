# Design system

The basis is the **WinUI 3 / Fluent 2** guidelines for Windows 11: Microsoft Learn, "Design and code
Windows apps" (Typography, Layout, Color, Materials/Acrylic, Iconography, Accessibility). This document
records the project's decisions on top of those guidelines. If they disagree, this document wins; where
it is silent, the WinUI 3 guideline applies.

> **Important: the toolkit.** WinUI 3 is a XAML framework (C#/C++). The application is written in Rust
> and uses `egui/eframe`. We **reproduce the WinUI 3 design language** (colors, spacing, typography,
> states, materials) through the `ui/theme` module, but we do not use XAML. Changing the GUI library
> (for example to Slint with a Fluent style) is a separate decision, made only with memory and CPU
> measurements (`docs/ARCHITECTURE.md`, section 5).

## 1. Principles

1. **Simplicity and minimalism.** One screen, one task. No decorative elements.
2. **Readability first.** Text is high-contrast and larger than "pretty"; density is moderate.
3. **Structure on a grid.** All sizes and spacing are multiples of 4 px; elements align to columns.
4. **Dark theme is the primary theme.** A light theme exists on the same tokens (section 3.1); the user
   switches it with the button at the bottom of the navigation pane, and the choice is saved.
5. **No effects for effect's sake.** No gradients, no glow shadows, no emoji. The only exception is
   the device stage of a device tile (section 8.1): a device is shown as an object in light.
6. **Color carries meaning.** Accent means action and selection; status always comes with an icon and text.
7. **Live response.** Every action gets visible feedback; states and transitions change smoothly and
   according to the laws of physics (section 9).
8. **Economy.** No visual device may require constant repainting; motion spends resources only while
   something is moving.

## 2. Prohibitions (hard)

| Not allowed | Use instead |
| --- | --- |
| Emoji and picture symbols in strings (checkmarks, warnings, Unicode arrows) | Icons from the Hugeicons set |
| Gradients (backgrounds, buttons, accent, bars) | A flat fill from a token. Only the device stage (8.1) has a radial glow and a contact shadow |
| Pure `#FFFFFF` and `#000000` in surfaces, strokes, accents | Close shades from the palette. Allowed for **text**, but the `text.*` tokens are used by default |
| 255/0 channels in saturated interface colors (`#FF0000`, `#00FF00`, `#0000FF`, `#FFFF00`, ...) | Muted shades (section 3) |
| Color literals in widget code | Only the tokens in `ui/theme/tokens.rs` |
| Large radii (> 8 px) and "pills" | Radii of 4 / 8 px (section 5) |
| Acrylic over the whole window and on every panel | The window backdrop is visible only in the navigation area; content is opaque (section 6) |

Exception: the color the user picks for lighting (swatch, preview) is data, not decoration, and is shown
as is.

## 3. Color

The palette is stored as tokens; all values are opaque and close to the WinUI 3 dark-theme tokens. The
white/black overlays that WinUI places over its background are replaced by precomputed shades, so that
pure `#FFFFFF`/`#000000` never appear.

### Surfaces

| Token | Value | Purpose |
| --- | --- | --- |
| `surface.sunken` | `#1A1A1A` | Recesses, input fields inside cards, log areas |
| `surface.base` | `#202020` | Window background (fallback color when acrylic is off) |
| `surface.card` | `#2B2B2B` | Cards, settings groups |
| `surface.card.hover` | `#303030` | Card on hover (if interactive) |
| `surface.overlay` | `#2C2C2C` | Drop-downs, tooltips, dialogs (acrylic fallback color) |
| `control.fill` | `#343434` | Button, input field at rest |
| `control.fill.hover` | `#3A3A3A` | Hover |
| `control.fill.pressed` | `#2F2F2F` | Pressed |
| `control.fill.disabled` | `#2A2A2A` | Disabled element |

### Strokes

| Token | Value | Purpose |
| --- | --- | --- |
| `stroke.subtle` | `#363636` | Dividers, card borders (decorative) |
| `stroke.default` | `#404040` | Borders of buttons and grouped elements |
| `stroke.control` | `#7F7F7F` | Borders of elements that are *invisible without a stroke* (input field, checkbox, radio button, switch in the off state). Contrast ≥ 3:1 |
| `focus.ring` | `#E6E6E6` | Focus ring, 2 px, 2 px offset |

### Text

| Token | Value | Purpose |
| --- | --- | --- |
| `text.primary` | `#F2F2F2` | Headings, main text |
| `text.secondary` | `#C8C8C8` | Explanations, captions |
| `text.tertiary` | `#A0A0A0` | Hints, secondary data. Do not place on `control.fill.hover` or lighter |
| `text.disabled` | `#6E6E6E` | Disabled text |
| `text.on-accent` | `#101010` | Text on an accent fill |

### Accent and statuses

| Token | Value | Purpose |
| --- | --- | --- |
| `accent` | `#4DB2E8` | Primary button, selected item, link, indicator |
| `accent.hover` | `#62BDEC` | Hover |
| `accent.pressed` | `#3F9FD8` | Pressed |
| `accent.subtle` | `#243F50` | Background of the selected navigation item and of a highlighted row |
| `status.success` | `#6CCB5F` | Success |
| `status.caution` | `#E0B84A` | Warning |
| `status.experimental` | `#FFA64D` | Small experimental caption below Back; light theme uses `#9C4800` |
| `status.critical` | `#E8808A` | Error, irreversible action |
| `status.info` | = `accent` | Information |
| `device.glow` | `#4DB2E8` | Light behind a hovered device model and its rim light (8.1); never text |
| `device.shadow` | `#0E0E0E` | Contact shadow under a device model, always drawn translucent (8.1) |

A status always consists of **icon + text + color**: color alone conveys nothing.

### Contrast check

Calculated per WCAG 2.x (luminance ratio) for the palette above:

| Pair | Contrast | Requirement |
| --- | --- | --- |
| `text.primary` on `surface.base` / `surface.card` | 14.6 / 12.6 | ≥ 4.5 |
| `text.secondary` on `surface.base` / `surface.card` | 9.7 / 8.5 | ≥ 4.5 |
| `text.tertiary` on `surface.base` / `surface.card` / `control.fill` | 6.2 / 5.4 / 4.8 | ≥ 4.5 |
| `text.tertiary` on `control.fill.hover` | 4.3 | do not use |
| `accent` on `accent.subtle` | 4.7 | ≥ 4.5 |
| `text.primary` on `accent.subtle` | 9.9 | ≥ 4.5 |
| `focus.ring` on `surface.card` | 11.3 | ≥ 3 |
| `text.on-accent` on `accent` / `accent.hover` | 8.0 / 9.1 | ≥ 4.5 |
| `accent` on `surface.base` / `surface.card` | 6.9 / 6.0 | ≥ 4.5 |
| `status.*` on `surface.card` | 7.0 / 7.5 / 5.3 | ≥ 4.5 |
| `stroke.control` on `surface.card` | ≈ 3.5 | ≥ 3 |

Rule: when any token changes, recompute the rows of the table. The "card background on window
background" pair (≈ 1.2:1) is deliberately soft: it is decoration, not an element boundary. The test
`both_palettes_meet_the_contrast_table` in `ui/theme/tokens.rs` checks every row for both themes, so a
token that breaks the table fails the build.

### 3.1 Light theme

Every token has a dark and a light value (`Token` in `ui/theme/tokens.rs`); widgets never pick a theme
themselves. Light values follow the WinUI 3 light tokens, with no pure `#FFFFFF`/`#000000`:

| Token | Light | Token | Light |
| --- | --- | --- | --- |
| `surface.sunken` | `#EEEEEE` | `text.primary` | `#1B1B1B` |
| `surface.base` | `#F3F3F3` | `text.secondary` | `#4F4F4F` |
| `surface.card` | `#FBFBFB` | `text.tertiary` | `#6B6B6B` |
| `surface.card.hover` | `#F6F6F6` | `text.disabled` | `#A3A3A3` |
| `surface.overlay` | `#F9F9F9` | `text.on-accent` | `#FAFAFA` |
| `control.fill` | `#FDFDFD` | `accent` | `#005FB8` |
| `control.fill.hover` | `#F6F6F6` | `accent.hover` | `#0A6AC4` |
| `control.fill.pressed` | `#F0F0F0` | `accent.pressed` | `#004C93` |
| `control.fill.disabled` | `#F5F5F5` | `accent.subtle` | `#D3E5F5` |
| `stroke.subtle` | `#E5E5E5` | `status.success` | `#0F7B0F` |
| `stroke.default` | `#D1D1D1` | `status.caution` | `#8F5500` |
| `stroke.control` | `#8A8A8A` | `status.critical` | `#C42B1C` |
| `focus.ring` | `#1F1F1F` | `status.info` | = `accent` |
| `device.glow` | `#62BDEC` | `device.shadow` | `#8C8C8C` |

The accent is darker in the light theme so that it keeps ≥ 4.5:1 on light surfaces. Mica is re-tinted
for the chosen theme. The application icon keeps its dark tile in both themes: it is identity, not
interface.

## 4. Typography

The font is the system **Segoe UI** (Segoe UI Variable if the library supports variable axes; otherwise
Segoe UI). We bundle no fonts of our own. The fallback is the built-in `egui` font.

| Role | Size / line height | Weight |
| --- | --- | --- |
| Caption | 12 / 16 | Regular |
| Caption Light | 12 / 16 | Light; secondary experimental status below Back only |
| Body | 14 / 20 | Regular (main text) |
| Body Strong | 14 / 20 | SemiBold |
| Subtitle | 20 / 28 | SemiBold (section heading) |
| Title | 28 / 36 | SemiBold (page heading) |

Rules: main text is never smaller than 14 px; Caption is only for secondary data. Left alignment; long
lines no wider than ~75 characters. A monospaced font is used only for hashes and identifiers
(`Cascadia Mono`, from the system).

## 5. Grid, spacing, radii

The base unit is **4 px**. Allowed steps: 4, 8, 12, 16, 24, 32, 48.

| What | Value |
| --- | --- |
| Page margins | 24 px |
| Content columns | 12 columns, 16 px gutter |
| Card inner padding | 16 px |
| Gap between cards | 12 px |
| Gap inside a group (label to field) | 8 px |
| Standard control height | 32 px |
| Navigation row height | 40 px |
| Minimum touch target | 32 × 32 px (40 preferred) |
| Navigation pane | 48 px collapsed / 240 px expanded |
| Control radius (buttons, fields) | 4 px |
| Card and overlay radius (menus, dialogs) | 8 px |
| Stroke | 1 px |
| Icon size | 16 px in dense places, 20 px by default, 24 px in headings |
| Device tile | Square, at least 224 px (`size::DEVICE_TILE_MIN`); 64 px kind icon when there is no model |

Alignment: text on the baseline, icon and text in a row centered; all card edges lie on column lines.
Non-standard values (7 px, 13 px) are forbidden.

### 5.1 Adaptive layout

The layout follows the window at every size and reflows live while the window is resized. Nothing has
a fixed page width, nothing stops in the middle of a wide window, nothing overflows a narrow one.

1. **Content fills the window.** Pages, cards, lists and headers stretch to the available width minus
   the page margins. A maximum width is allowed only for **running text**: paragraphs wrap at
   `size::READABLE_WIDTH` (~75 characters) *or the available width, whichever is smaller*. A text cap
   never widens its container.
2. **Breakpoints are tokens** (`ui/theme/tokens.rs`, `breakpoint`), modeled on the WinUI size classes:

   | Token | Measured on | Value | Effect |
   | --- | --- | --- | --- |
   | `NAV_EXPANDED` | window | 1008 px | Below it the navigation pane collapses to icons (48 px) |
   | `PAGE_COLUMNS` | page content | 960 px | From it the cards of a page stand in two columns |
   | `SECTION_COLUMNS` | card content | 560 px | Below it the side-by-side parts of one card stack |

   Content breakpoints are measured on the space a part actually gets, not on the window, so the same
   component works in a column, in a narrow window and next to an expanded pane.
3. **Reflow, not scaling.** Wide layouts add columns (`pages/layout.rs`, `columns`); narrow ones stack.
   Controls keep their size from section 5; text is never shrunk to fit.
4. **Changes between layouts are animated** where geometry moves (the pane width uses
   `spring.standard`); a reflow of cards follows the window directly, without lag.
5. **No layout flips in the middle of a motion.** While the pane collapses or expands, content
   breakpoints are decided on the width the content *will* have when the pane settles
   (`settled_width`), so columns switch once, at the start. Labels of the pane fade with its width and
   slide under its edge instead of disappearing at a threshold.
5. **Check every page at three widths**: the minimum window (720 px), the intermediate (1040 px) and a wide
   monitor (1600 px). `--review DIR` renders each scene at all three; the result is attached to the
   change description.

### 5.2 Device workspace

The initial window is 1360 x 840 logical pixels. Device pages reserve the available viewport instead of
joining the page scroll. Opening a device temporarily compacts navigation to 48 px; returning restores
the previous list preference. A large model leads, followed by identity, battery, compact USB/firmware
facts and capability actions on the trailing side. USB identifiers, firmware and release codes live
in a hover tooltip on the connection status, including its icon. Battery stays visible separately.
There is no duplicate About card or standing preview-without-writing caption.

**Device capability actions.** Lighting, encoder and every future capability entry point use the
shared `ActionTile` widget: medium square tiles, never small horizontal buttons. Each tile has an
8 px corner radius, a centered 40 px icon from the icon set and a wrapping 14 px label below it, with each line centered.
All actions have equal visual weight and align to a regular grid with 12 px gaps. Tile sides reflow
between 112 and 144 px from the size tokens; the grid derives its column count from available width.
Keep square proportions when resizing, use additional rows for more actions and allow only the
action grid to scroll if it exceeds the remaining height. Do not stretch tiles into long rows.
Hover, press and keyboard focus use the shared spring interaction and focus ring. This pattern is
mandatory for all current and future devices and capability actions; ordinary editor controls,
navigation and explicit Apply/Select buttons retain their control styles.

Lighting and encoder actions open a bottom island with choices: title, visual sample and a
Settings button. Choosing a mode centers the enlarged model, removes identity and targets the camera
at the relevant part. Settings restores a trailing inspector and moves the model aside. Camera pose,
zoom, focus, island travel and panel width use critically damped `spring.workspace` transitions,
without bounce. Resolve the model's fitted square and center from the destination layout first, then
animate them directly. Never fit the model to an intermediate animated panel rectangle: its expanding
width and shrinking height can reverse the model's scale even with critical damping. Measure notice
space against the destination width too. Encoder framing also interpolates instead of switching size on entry. Overshoot is
reserved for opening a device from the list. Reduced motion drops spatial
travel through the shared motion system. Selection also has a check, border and title.

`STUDIO_PANEL` is 300 px, capped at 43% of available width. Below `SECTION_COLUMNS` the inspector
stacks. The inspector must never scroll: all controls and actions fit its available height. Reserve complete control rows first and scale the wheel into the remaining space. At short heights place the wheel beside the controls, with hardware actions wrapping below. Below `STUDIO_SHORT_HEIGHT` the island is 184 px instead of
220 px, including its dedicated close-button header. The page never grows to fit editor contents.
Experimental devices show a small Segoe UI Light caption below Back, in the orange experimental status color,
without a badge, icon or background. Do not repeat it in the device identity or editor messages.
The top Back action and Escape unwind inspector, editor and device in that order. The island's close
button remains available and returns directly to the device overview. Backup actions live in Settings,
outside the device workspace. The settings action is inside each card at its
bottom trailing corner; the title stays inside the same card above the sample.

Visual bindings in `assets/models/stages.toml` associate asset node names with key input and encoder
focus; they confer no hardware support. Solid/Wave/Reactive are model previews. Wave runs for 1.6 s
after selection and stops; held keys and short release tails also stop when settled. Native continuous
effects preview automatically while focused; no separate demonstration action is needed.
Input comes only from the focused window and is suppressed during text entry. Encoder assignments are
local placeholders. Native lighting drafts update the preview immediately; only explicit Select or Apply writes to the device. Page opening, card selection and sliders never write.

Drivers with native lighting programs replace the three preview choices with a horizontally scrollable
catalog supplied by the lighting capability. The inspector exposes only supported brightness, speed
and custom-color controls. Preset selection and sliders edit a local draft; Apply is always explicit. A card click changes the preview and accent outline only. A separate checkmark identifies the mode last read from the connected device, independent of the preview selection. Put Select beside Settings on unapplied native program cards; omit Select on the device-confirmed current card. Apply remains available in its inspector for edited settings. Select commits that card draft; Apply in the inspector commits the inspected draft through the same path. Move the checkmark only after successful device confirmation. Pending or failed writes keep the last confirmed marker; static/offline catalogs have no applied marker. Local preferences never supply the applied state. Native drafts never enter an automatic write queue. Background battery reads must not gray or flash Apply. An explicit commit may wait for an in-flight telemetry worker; the completion event dispatches it once. Slider rows and commit actions use stable widget identities independent of preceding controls.
Native model demonstrations are labeled illustrative, since firmware owns the actual animation.
The owner's 2026-10-29 correction supersedes the finite-demo rule for native continuous programs:
they animate while the lighting editor is focused, stop when closed/unfocused or reduced motion is
enabled, and key-triggered programs stay dark without actual key events. Hardware id 7 was explicitly
verified by the owner as key-triggered. Spatial timing remains illustrative until separately verified.
Native preset cards retain the original rectangular surfaces, selection borders/checks, interaction
springs and trailing Settings action. The hue wheel and compact saturation/brightness/speed controls fit without scrolling for every preset. Custom color is the default for programs supporting it; rainbow-only
hardware modes retain their palette restrictions. Named native effects share distinct illustrative
spatial/temporal models between the cards and 3D stage. Static/reactive states cache at rest; native
continuous modes run only in the focused editor as requested by the owner.
Brightness scales emission through the shared spring system without dimming the hue rings or plastic.
When a driver supplies static preview metadata but the connection has no lighting capability, keep
the same program catalog and controls visible with illustrative values, an explicit connection hint
and disabled Apply. Offline catalog pages use this metadata too and never create a device session.

### 5.3 LED color and preview fidelity

The lighting picker contains concentric hue rings: one for Solid/Reactive and three for Wave. Every
ring is independent and separated by an 8 px inset. Its marker follows a release spring; the outlined
center previews the active color. Keyboard Left/Right changes hue and Up/Down selects a ring. This is
LED color selection: value stays at 100%; saturation ranges from white to vivid color, allowing pastel shades.
Hue and radial edge coverage are evaluated per texture pixel, at native display density with a 2x
minimum. Both edges must stay smooth; the wheel texture is cached independently of the moving marker.
At short heights the entire wheel scales to the available inspector height; duplicate headings are
omitted. Saturation updates the hue rings, markers and center together at full LED value; 0% is white, so there is no separate White button. Show saturation as a whole percentage. The ring cache includes saturation and updates only when its inputs change. HSV selection operates directly on the RGB channel byte values: 50% cyan saturation is 128/255/255, with no additional display-gamma encoding applied to the device bytes. The ring, marker, center and transmitted color share this conversion; device color calibration is a separate concern.

Lighting sliders use a 12 px circular thumb with a dark-gray center and a 2 px theme-accent outline, on a 4 px rail; retain a 24 px interaction row and keyboard/numeric input. Labels, tracks and values align in columns. The normal editor has no standing information strip. Show at most one current connection blocker or failure below the model and above the island; reserve its measured height. Loading belongs on the clicked button and the confirmed mode has its card checkmark. Do not add routine success text or explanations of persistence, backups, protocol validation, preview algorithms or missing rollback to the workspace. These rules apply at 720/1040/1600 px, the 520 px minimum height and in both themes.

Device textures retain their native resolution up to 4096 px. The keyboard's 1536 px atlas must never
be halved for a workspace; tiles reuse the same parsed asset. Texture coordinates are interpolated
with perspective correction. Keycap plastic keeps its base color: emission is restricted to legend
apertures and under-cap socket geometry, with distance/normal-dependent local diffuse spill. Bloom
uses depth-tested visible emission only, so bright white plastic does not glow. This approximates a
backlit material; it is not a claim of ray-traced lighting or measured hardware behavior.
The GPU stage uses roughness/metallic shading, a filtered shadow map, contact shading and restrained
HDR bloom, with 4x MSAA and mipmapped anisotropic texture sampling. Studio lighting must reveal cap
sidewalls and gaps without bleaching their tops. Pointer tracking remains active while editing, at
28% of the overview's angular range, so the close camera remains stable and responsive.

## 6. Materials: acrylic

Acrylic is a translucent blurred Windows 11 material. We use it **sparingly**.

- **Allowed:** the window backdrop visible through the transparent area of the side navigation. System
  acrylic (a DWM backdrop) blurs what is *behind the window*; blurring content inside the window (behind
  a menu or a dialog) is not possible with system means. Therefore menus, tooltips and dialogs are drawn
  with the opaque `surface.overlay`, not with acrylic.
- **Not allowed:** acrylic under the main content, under text fields and tables, layered acrylic,
  acrylic on cards.
- **The main content area is always opaque** (`surface.base`/`surface.card`): text readability matters
  more than the effect.
- **A fallback is mandatory:** if "transparency effects" are off in Windows, the window is inactive or
  battery saver is on, the surface is filled with an opaque token (`surface.base` / `surface.overlay`).
- **Implementation:** DWM composes one native Acrylic material behind the caption and the
  transparent navigation area. `transport/acrylic.rs` extends the DWM frame into the client area
  and sets `DWMWA_SYSTEMBACKDROP_TYPE` to `DWMSBT_TRANSIENTWINDOW`. The caption's immersive dark
  mode follows the application palette, so caption buttons remain readable. Windows owns material
  tint, opacity and inactive-window fallback; the application does not impose an accent-color fill.
  The DX12 swapchain uses a DirectComposition visual with premultiplied alpha. The HWND's legacy
  blur region must cover the entire window with alpha enabled (`DWM_BB_ENABLE | DWM_BB_BLURREGION`,
  `fEnable = TRUE`, null region); disabling it exposes an opaque redirection bitmap after repaint.
  On the frame after eframe's first presentation, initialize the HWND client redirection surface
  to zero using the stock black GDI brush and retain that class brush for exposed resize regions.
  DWM attributes and a transparent swapchain alone do not clear the initially white native bitmap.
  This is initialization of native glass, not application backdrop rendering or a GPU readback.
  Content remains opaque. Unsupported DWM composition uses `surface.base`.
- **Correction (2026-10-29):** replaced winit's empty legacy region with a full-window alpha region
  instead of disabling it. Caption, navigation icons and fallback colors now follow the selected
  application theme together, even when Windows uses the opposite theme. Native review covers both
  palettes, 720/1040/1600 px, forced repaints and minimize/restore. Native Acrylic opacity remains
  system-controlled; custom opacity is not claimed.
- **Ordinary-launch regression (2026-10-29):** native review previously resized/repainted the window
  before capture, concealing the white HWND client bitmap. Acceptance now includes launching the
  packaged EXE without arguments and capturing the expanded navigation before any resize, repaint
  request or material adjustment from the capture helper.
## 7. Icons

The only source is the **Hugeicons** set (hugeicons.com), one style across the whole application
(Stroke Rounded with a 1.5 px line is recommended). Styles must not be mixed and icons from other sets
must not be blended in.

- Icons are stored as **embedded vector outlines of only the glyphs in use** (Hugeicons Free 4.3.5,
  Stroke Rounded, `ui/icons/glyphs.rs`) and referenced through `enum Icon` in `ui/icons/mod.rs`. They
  are flattened once and stroked at 1.5 px on the 24 px grid. A font subset is not used because font
  glyphs are fills, while the set is stroke-based. An icon's color is a text or status token.
- An icon next to text has the same color as the text, except for statuses.
- An icon without a label gets a tooltip and an accessible name.
- **License:** before adding to the repository, check the terms of exactly the set and format in use
  (free and paid sets differ) and add an entry to `THIRD-PARTY.md`. Do not commit icons with an
  unsuitable license.
- Emoji and Unicode symbols instead of icons (`✓`, `⚠`, `→`) are forbidden. Ordinary punctuation, such as
  the `·` separator, is allowed.

## 8. Components and states

Every interactive element has states: rest, hover, pressed, disabled, focus (keyboard), selected. All
states are defined by the tokens in section 3; we do not introduce new colors for a single screen.

| Component | Rules |
| --- | --- |
| Primary button | `accent` + `text.on-accent`; one per screen/card |
| Regular button | `control.fill` + `stroke.default`, text `text.primary` |
| Dangerous action | Regular button with an icon and `status.critical` text; confirmation as a separate step |
| Input field | `surface.sunken`, `stroke.control` border, a 2 px `accent` bottom line on focus |
| Switch, checkbox | Off: `stroke.control` border; on: `accent` fill. The switch track is 40 × 20 with a 4 px radius and a 12 px thumb (no pill shape, section 2) |
| Card | `surface.card`, radius 8, `stroke.subtle` border, no shadow |
| Navigation | On the left; the selected item gets `accent.subtle` and a 3 px vertical `accent` bar |
| Dialog | `surface.overlay`, radius 8, background dimming with `surface.sunken` at 60% opacity |
| Status badge | Icon + text, `status.*` color, `surface.sunken` background |
| Tooltip | Appears after ~500 ms, `surface.overlay`, Caption |

Every state changes smoothly under the rules of section 9; there are no "instant" switches.

### 8.1 Device tiles (fixed for every device)

Found devices are shown as **square tiles**, never as long flat rows. The view is the same for every
device and every kind (keyboards, mice, lighting, fans, ...); only the model inside changes.

- **The list keeps itself current.** Devices appear and disappear as they are plugged in and out;
  the user never has to start a search. The header has one regular (not primary) button,
  "Refresh", with the search icon: a forced search. While a search runs it shows the busy state.
- **Recognized but not enabled.** A device whose driver is experimental and off still gets its
  tile, with a `status.caution` "Driver off" badge instead of "Experimental". Its page explains
  why nothing is sent to it and offers "Open settings"; turning the setting on searches again.
- **Grid.** Tiles stand in as many columns as fit (`size::DEVICE_TILE_MIN` = 224 px each, 12 px gap);
  the side is the column width, so the grid fills the content width at any window size. The column
  count is decided on the settled width (section 5.1).
- **Stage.** The upper part of the tile is a stage with a 3D model of the device: front, from above,
  turned slightly to the left. The name (Body Strong) and a caption "kind · vendor · driver" (Caption)
  sit at the bottom; status badges (for example "Experimental") in the top left corner.
- **Light.** The model is lit by a warm key light from the upper left and a sky/ground fill; a soft
  `device.shadow` ellipse lies under it.
- **Hover.** The model rises (`motion::TILE_RISE`, 8 px, plus a small 3D lift), turns toward the
  pointer (up to `TILT_YAW` 0.24 rad and `TILT_PITCH` 0.12 rad), the stage behind it glows with a
  radial `device.glow` spot and the model gets a rim light of the same color. The contact shadow
  shrinks and fades as the model rises. The card fill flows to `surface.card.hover` as on any card.
- **Motion.** Lift and glow use `spring.standard`, the turn uses `spring.tilt` (light inertia). All
  of it is interruptible and returns to rest when the pointer leaves; nothing moves on its own.
- **No model.** A keyboard without its own model uses the orange generic keyboard mockup. Other device kinds show their kind icon (64 px, `text.secondary`) on the same
  stage with the same shadow, glow, lift and a small parallax shift instead of the turn.
- **Where models come from.** A model is data: a profile names it (`visual = "..."`,
  `docs/ADDING-A-DRIVER.md`) and the file lives in `assets/models/`. The interface never picks a model
  by brand or driver.
- **Cost.** The wgpu stage draws only when pose, size or lighting changes, at full resolution with
  4x MSAA. The interface samples the GPU texture directly; there is no native-frame CPU readback.
  Static meshes and mipmapped atlases upload once per visual. HDR, MSAA, shadow, depth and bloom
  work textures are released at rest or when hidden; the final image remains cached. An idle tile is
  one textured rectangle. With Windows animations off the model does not turn or rise; only the glow
  fades in. Bounded optical bloom belongs to the 3D stage and does not blur the interface backdrop.

### 8.2 Device detail header

The tile expands into the full content area and the navigation pane collapses to its compact form.
The page uses one shared interactive model stage, with the same pose springs and texture identity
as the source tile. The enlarged model occupies the main workspace; name, driver, connection,
battery and actions share the trailing panel. The technical passport does not duplicate these fields
below the stage. Editing replaces this panel and introduces the bottom island as described in 5.2.

Battery data comes from driver battery capabilities and refreshes on adaptive deadlines while the real window runs. Display one row with proportional icon fill, percentage and power/charging status; omit discovery-time wording.
Missing data is explicit; the interface does not infer a percentage or a connection transport.

### 8.3 Interactive model quality standard

This is the product standard for all device models, not a keyboard-specific demonstration. New assets,
new device editors and renderer changes must preserve it. Sections 5.2–5.3 define the surrounding
workspace and picker. Implementation and measured costs live in `ARCHITECTURE.md` and
`GPU-PERFORMANCE.md`; historical measurements are not universal performance guarantees.

**Composition and camera.** The device is the main subject and must remain large and readable within
the available viewport. It must not force page scrolling. Keep identity and actions in the trailing
panel in overview. Opening a visual editor removes identity, centers/enlarges the model and smoothly
focuses the relevant physical region. Opening settings moves the model aside and uses that panel for
the inspector. Closing the island restores overview; the top Back button unwinds one level at a time.

| State | Required model behavior |
| --- | --- |
| Overview | Large three-quarter view with visible thickness; full restrained pointer tracking |
| Lighting editor | Closer view of luminous surfaces, with readable legends and visible gaps/sidewalls |
| Encoder or other part editor | Focus and zoom on the bound physical part; retain enough context to recognize it |
| Settings inspector | Preserve the selected effect and camera focus while making space for controls |
| Close or interrupted transition | Return or reverse continuously, preserving spring velocity |

Zoom must not disable pointer tracking: use 28% of the overview's angular range while editing, with
the same tilt springs. Do not flatten the camera into an orthographic-looking top view that hides
thickness. Current keyboard reference values are lighting zoom 1.12/pitch 0.88 rad and encoder zoom
2.2/pitch 0.85 rad. These are a reference for this asset, not mandatory framing for differently shaped
devices; bind focus and framing through visual metadata rather than manufacturer checks. Camera,
focus, zoom, island and panel transitions use interruptible springs and respect reduced motion.

**Materials and detail.** Judge quality at the largest supported editor view, not only on a small
tile. Legends must be correct, oriented correctly, sharp and readable. Preserve native source detail;
do not halve textures or lower rendering resolution during camera motion. Enlarging a blurry bitmap
does not create detail. For bound key surfaces, the reference implementation uses a separate 2048 px
atlas with 192 px cells and outline-rasterized legends, with a matching emission mask. Other assets
may use suitably detailed authored textures or equivalent outline rendering. Keep source provenance;
do not invent product markings. Missing system fonts may retain the source asset as a fallback, but
that fallback is not evidence that a new asset passes close-up quality review.

Plastic albedo, surface roughness and metallic parts must respond independently to lighting. Replace
baked photographic highlights on relit surfaces rather than stacking a second lighting model over
them. Preserve the device's colors, bevels and shape. Exposure must retain light-plastic detail and
color differences instead of washing everything toward white. Dark crevices, contact shading,
filtered shadows and controlled reflections must make depth readable without crushing the image.

**Lighting effects.** Emit light from bound sockets, gaps, translucent legends and other genuinely
luminous geometry. Preserve opaque material colors; a whole-texture recolor is not a lighting effect.
Use local spill and restrained HDR bloom from visible emission only. White plastic must not become a
glowing source. The reference stage uses roughness/metallic shading, shadow mapping, contact shading,
4x MSAA and mipmapped anisotropic sampling. Equivalent replacements must preserve or improve the
visible result. Do not describe this approximation as measured hardware output or ray tracing.
Mode/color changes and focused-window key presses must affect the corresponding model parts.
Maintain the full-brightness, smooth concentric LED hue rings specified in 5.3.
The opened device uses a directional studio rig with reduced fill, stronger contact shading and a
subtle model-edge vignette; the rig blends with the shared-surface reveal. A soft stage shadow grounds
the case without darkening the controls. LED bloom must prefilter thin luminous gaps before
downsampling and use the same blur footprint on both axes. Reactive waves retain a bright front and
a smooth, finite wake in model space; decay scales linear radiance rather than the selected sRGB
color. Camera movement must not leave screen-space ghost copies of the device.

**Cost and separation from hardware.** Use the GPU and sample native output textures directly without
per-frame CPU readback. Upload static geometry/materials and build legend atlases once per visual.
Cache the resolved image; render again only for changed pose, size or effects. After springs, key
release tails and finite effect demonstrations settle, stop requesting animation frames and submitting
model work. Release large HDR/MSAA/depth/shadow/bloom work targets at rest or when hidden. No permanent
animation threads, polling or autonomous demonstrations. Visual bindings grant no hardware support;
passive preview rendering never writes to a device. Explicit user edits to native lighting enqueue coalesced configuration writes with backup/read-back. Offline previews never access hardware.

**Acceptance.** Review overview, lighting, part zoom and settings at 720/1040/1600 px, including the
minimum height, both themes, pointer movement while zoomed, close/reversal and reactive key feedback.
Inspect a large screenshot for readable undistorted legends, smooth edges, preserved material color,
visible sidewall depth and light confined to its sources. Verify idle repaint/submission counters stop
and temporary targets are released. Record active-frame and idle resource measurements when rendering
cost changes, distinguishing offline readback timings from native GPU presentation. Codec tests and
screenshots do not establish physical-device behavior. Reject regressions even when tile-sized images
look acceptable.

### Application shell and brand neutrality

The application does not belong to a single manufacturer, so the shell is built around the device, not the
brand:
- **Navigation on the left:** "Devices" (tiles of everything found, section 8.1), "Diagnostics",
  "Settings"; an opened device gets its own page.
- **The application is for users, not for developers.** It configures devices; it does not research
  them. Tools for adding devices (firmware or package analysis, protocol labs) are not part of the
  interface: static package analysis exists only as the developer command `--inspect`, and
  "Diagnostics" only checks the connection and saves a passive report a user can send to developers.
- **A device page** consists of sections by its *capabilities* ("Lighting", "Keys", "DPI", ...): only
  supported ones are shown, the rest are not drawn.
- The interface does not use manufacturers' brand colors, logos or fonts. A brand name appears only as
  device data (from a profile) and never as part of the styling.
- The application's own name and icon come from a single place (`app/identity.rs`), so renaming does not
  touch the screens.

## 9. Motion and physics

The interface is alive: **every user action gets visible feedback**, and every state change and every
transition between pages and windows is smooth motion. Motion obeys physics: elements have mass, velocity
and inertia instead of a fixed "from-to" animation.

### 9.1 Principles

1. **Immediate response.** The reaction to hover, press and focus starts in the same frame as the event.
   An operation on a device (applying a color) first gives an instant visual answer (the button looks
   "pressed", an indicator appears), and the result is shown as a separate step.
2. **State, not a script.** An animation is a value moving toward a target. There is no "play 200 ms"
   script: the target changes and the value eases toward it.
3. **Interruptibility.** Any animation can be interrupted at any moment (quick back-and-forth clicks, a
   switch to another page in the middle of the previous one). The new target inherits the *current
   position and current velocity*: no jumps, stops or jerking.
4. **Physicality and inertia.** A body accelerates instead of starting at full speed, and decelerates
   smoothly at the target. After a gesture (dragging a slider, scrolling) motion continues at the
   velocity at which the user released it and decays under friction.
5. **Restraint.** Motion supports meaning, it does not entertain: small displacements, zero or near-zero
   overshoot. Minimalism (section 1) applies here too.
6. **Economy.** Animation spends resources only while something is moving. When all values have reached
   their targets, repainting stops completely (`docs/ARCHITECTURE.md`, section 5).

### 9.2 Model: the spring

The main tool is a **damped spring**, not a time-based cubic curve. It provides acceleration and
deceleration by itself, keeps velocity when the target changes, and behaves consistently over any
distance.

A spring is defined by two numbers: the period `T` (seconds; smaller means faster and stiffer) and the
damping ratio `ζ` (1.0 means no overshoot; < 1.0 means a small overshoot). Stiffness and damping follow
from them: `ω = 2π/T`, `k = ω²`, `c = 2ζω`.

| Token | T, s | ζ | To 95% of target | Used for |
| --- | --- | --- | --- | --- |
| `spring.fast` | 0.18 | 1.00 | ≈ 0.14 s | Hover, press, focus, row highlight, an element's color change |
| `spring.standard` | 0.30 | 1.00 | ≈ 0.23 s | Switches, item selection, expanding a group, popups |
| `spring.page` | 0.40 | 0.95 | ≈ 0.30 s | Transitions between pages and windows |
| `spring.workspace` | 0.40 | 1.00 | ≈ 0.30 s | Device editor camera, pointer tracking, framing, island and inspector; no overshoot |
| `spring.device_reveal` | 0.52 | 0.85 | Overshoot about 0.6% | Only the shared model during device opening; return uses `spring.page` |
| `spring.release` | 0.32 | 0.80 | ≈ 0.3 s, overshoot ≈ 1.5% | Sliders, the color-picker marker, return after a gesture |
| `spring.tilt` | 0.50 | 0.90 | ≈ 0.4 s, overshoot ≈ 0.2% | A device model turning toward the pointer (8.1) |

New values are introduced only as tokens in `ui/motion/tokens.rs`. Overshoot > 3% and `ζ` < 0.7 are not
allowed: the interface must not "bounce" like a cartoon.

**Computation.** A spring is computed with an analytic formula or an implicit step that is stable at any
time step. The step is clamped (`dt ≤ 1/30 s`) so that a frame stall does not "blow up" the calculation.
Spring state is position, velocity and target. Stop condition: both the deviation and the velocity are
below a threshold (≈ 0.1 px, or 0.5% for dimensionless values); then the value is fixed at the target
and the animation is removed from repainting.

### 9.3 What moves and how

| Event | Motion |
| --- | --- |
| Hover over a button, row, card | Smooth transition of fill/stroke color to `*.hover`, `spring.fast` |
| Hover over a device tile | The model rises and turns toward the pointer, the stage glows (8.1); `spring.standard` and `spring.tilt` |
| Press | Scale to 0.97 and `*.pressed` color, `spring.fast`; release returns taking velocity into account |
| Keyboard focus | The `focus.ring` fades in (opacity 0 → 1), `spring.fast` |
| Switch, checkbox | The thumb/check travels to the new position, color flows, `spring.standard`; the thumb stretches slightly while pressed |
| Navigation item selection | The 3 px `accent` bar **travels** from the old item to the new one instead of fading out and flashing; `spring.standard` |
| Theme switch | One blend value moves on `spring.standard`; every token mixes its dark and light values by it, so the whole interface flows to the new theme together. The sun and moon icons cross-fade and turn a quarter turn |
| Navigation pane collapse | Width on `spring.standard`; labels fade with the width (section 5.1) |
| Expanding a group / list | Height and opacity grow toward the target, `spring.standard`; content does not jerk over neighbors |
| Menu, tooltip | Appear with opacity 0 → 1 and an 8 px shift from the anchor, `spring.standard`; closing is faster, `spring.fast` |
| Dialog | The dimming fades in; the dialog window scales 0.96 → 1 with opacity, `spring.standard` |
| Page transition | Section 9.4 |
| Slider, color picking | The marker follows the cursor with light inertia; on release, `spring.release` with the gesture's initial velocity |
| Applying a color (a device operation) | The button shows busy at once; on completion the indicator smoothly changes into a status icon |
| Changing the selected lighting color in the preview | The swatch color flows to the new one instead of jumping |
| Appearance and disappearance of status badges and notifications | Opacity and an 8 px shift, `spring.standard`; several badges slide smoothly to make room |
| Scrolling list/area | Kinetic scrolling: after a gesture velocity decays exponentially, `v(t) = v₀·e^(−t/τ)`, `τ ≈ 0.3 s`; stops at `|v| < 20 px/s`. No rubber-band edge (it is not Windows behavior) |

Only cheap things are animated: opacity, color, offset and render scale. A container's size is animated
only if the content would otherwise "jump" (expanding a group).

### 9.4 Transitions between pages and windows

A transition is always **directional** and shows the relationship between screens. The style is chosen by
meaning, as in WinUI 3:

| Meaning of the transition | Style | Motion |
| --- | --- | --- |
| Peer pages (navigation items) | Entrance (`Entrance`) | The new page fades in and rises 16–24 px from below; the old one fades out in place |
| Device tile to device page | Shared surface | Tile bounds expand to the content bounds; one model stage travels to the header; text and settings reveal late in the motion |
| Going deeper (other details, a report) | Drill In (`Drill In`) | New page: scale 0.96 → 1 and opacity; the old one leaves fading |
| Going back | Reverse drill in | Mirrors the forward one |
| Window/dialog | Scale + opacity | Section 9.3 |

Device navigation uses `spring.page` for the surface and a separate `spring.device_reveal` for the
model stage during opening. The latter retains its small overshoot; its progress is not clamped to one.
Return uses `spring.page` for both. The selected tile retains its chrome and labels but does not draw
a second model. The source list participates only during travel and is
removed from rendering when the page settles. Escape, Back and the Devices navigation item retarget
both springs, preserving their positions and velocities. The source list uses explicit dimming
(`motion::DEVICE_LIST_DIM`) instead of disabled-state opacity, returning continuously to normal
brightness. During the last `motion::DEVICE_TILE_HANDOFF` fraction of travel, the surface opacity
follows smoothstep to expose the tile labels before the final handoff. Detail scrollbars are hidden during travel to avoid a
moving clipping edge becoming a spurious scrollbar. With Windows animation disabled, geometry stays
at its destination and only the reduced opacity transition remains.

Rules:
1. The old page leaves faster than the new one arrives; at some moment both are visible, but the overlap
   never produces unreadable text (the old page's opacity drops first).
2. `spring.page` for arrival, `spring.fast` for departure.
3. Input is never blocked: while a transition runs, elements already in place respond to hover and click.
4. A quick repeated switch interrupts the transition (section 9.1, item 3), keeping velocity.
5. Page state (scroll position, entered fields) survives leaving and returns without a reset animation.

### 9.5 Limits and accessibility

- **Duration.** Any motion is visually complete within ~0.4 s. Exceptions: the progress indicator and
  kinetic scrolling.
- **System setting.** If animations are turned off in Windows (`SPI_GETCLIENTAREAANIMATION`, "Show
  animations in Windows"), displacement and scaling are dropped; a short (≈ 100 ms) opacity and color
  change remains. Functionality does not change.
- **No endless animations**, except the busy indicator, and only while an operation runs.
- **No flashing or shaking.** No frequent flashes, no oscillation with ζ < 0.7, no constant wobble.
- **Load.** During an animation repainting follows the display refresh rate; after it stops, it is zero.
  The number of simultaneously active springs is small (dozens), and one spring costs a few arithmetic
  operations; the frame budget with animation is ≤ 4 ms CPU.
- **Window minimized or hidden**: animations are suspended, and on return the values jump to their
  targets.

## 10. Accessibility

- All functions are reachable from the keyboard; the Tab order matches reading order, top to bottom and
  left to right.
- The focus ring is visible on any background (`focus.ring`).
- Windows scaling (100–200%) and large text: sizes are set in logical pixels, and the layout does not
  break when text grows by 25%.
- Information is never conveyed by color alone (icon + text).
- Error texts name the cause and the action: "Close the vendor software and try again", not "Error 0x5".

## 11. Design checklist for an interface change

- [ ] Tokens only; no color or size literals outside `ui/theme`.
- [ ] No gradients, emoji, `#FFFFFF`/`#000000` (except text), or 255/0 channels in styling (the device
      stage of section 8.1 is the only place with a glow and a shadow).
- [ ] Spacing and sizes are multiples of 4; radii are 4 or 8.
- [ ] The layout is adaptive (section 5.1): content fills the width, only running text is capped, the
      `--review` renders at 720, 1040 and 1600 px show no clipped, overflowing or stranded parts.
- [ ] Icons only from Hugeicons, one style, through `enum Icon`.
- [ ] Contrast recomputed if the palette changed.
- [ ] The window backdrop is visible only in the navigation area, and an opaque fallback exists.
- [ ] Every new action and state change has feedback under section 9, with values only from `spring.*` tokens.
- [ ] An animation can be interrupted at any moment without a jump; input is not blocked during animation.
- [ ] With Windows animations turned off, the interface remains usable.
- [ ] The idle window does not repaint (checked in Task Manager), including after animations.
- [ ] A before/after screenshot is attached to the change description.


Lighting sliders use compact accent-outlined circular handles with dark-gray centers. Speed 0 means very slow motion; preview phase advances at a positive rate without resetting when speed changes.


### Dedicated backup utility

At the owner's request, the separate backup executable uses a minimal, independent window with a device selection control, archive actions and status. It does not include Studio navigation, 3D previews, lighting or encoder editors. The main application's discovered-device inventory retains the square 3D tile standard; the Settings backup selection control is an operation target selector, not a replacement inventory. Its first Settings card always exposes Save and Restore affordances, disabled with a clear reason when unavailable. Partial snapshot coverage must stay explicit.

### 8.4 Keyboard cosmetic finishes

BY Tech profiles use the light purple finish by default: ivory housing and alphanumeric keys,
lavender modifiers, navigation and F5-F8, with pale blue Escape, Enter and Space. The reference
photographs supplied by the owner define the color grouping. Two small circular swatches,
orange then light purple, sit below the keyboard in its overview. Selection is saved locally
and applies consistently to tiles and the workspace; it changes appearance only, never HID state.
Selection rings animate using the shared springs.

SONiX profiles use an ivory-white finish with thin dark flowing contour lines printed on the
keycaps and readable dark legends. The print belongs to the plastic atlas, not the emission mask.
The orange keyboard is also the generic visual mockup for future keyboards lacking a model;
it does not establish their geometry, protocol, capabilities or hardware compatibility.
Non-keyboard devices without models retain their kind icon. All finishes retain section 8.3.
