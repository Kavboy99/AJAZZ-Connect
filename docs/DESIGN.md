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
4. **Dark theme is the primary theme.** A light theme may come later, on the same tokens.
5. **No effects for effect's sake.** No gradients, no glow shadows, no emoji.
6. **Color carries meaning.** Accent means action and selection; status always comes with an icon and text.
7. **Live response.** Every action gets visible feedback; states and transitions change smoothly and
   according to the laws of physics (section 9).
8. **Economy.** No visual device may require constant repainting; motion spends resources only while
   something is moving.

## 2. Prohibitions (hard)

| Not allowed | Use instead |
| --- | --- |
| Emoji and picture symbols in strings (checkmarks, warnings, Unicode arrows) | Icons from the Hugeicons set |
| Gradients (backgrounds, buttons, accent, bars) | A flat fill from a token |
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
| `status.caution` | `#E0B84A` | Warning, experimental |
| `status.critical` | `#E8808A` | Error, irreversible action |
| `status.info` | = `accent` | Information |

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
background" pair (≈ 1.2:1) is deliberately soft: it is decoration, not an element boundary.

## 4. Typography

The font is the system **Segoe UI** (Segoe UI Variable if the library supports variable axes; otherwise
Segoe UI). We bundle no fonts of our own. The fallback is the built-in `egui` font.

| Role | Size / line height | Weight |
| --- | --- | --- |
| Caption | 12 / 16 | Regular |
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

Alignment: text on the baseline, icon and text in a row centered; all card edges lie on column lines.
Non-standard values (7 px, 13 px) are forbidden.

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
- **Implementation:** the blur is done by the system DWM: the window backdrop type is set with the
  `DWMWA_SYSTEMBACKDROP_TYPE` attribute (Windows 11 22H2 and later; Acrylic or Mica) through `windows-rs`
  or the `window-vibrancy` crate, and the window is created with a transparent background. The WinUI 3
  guidelines recommend Mica for a long-lived window and acrylic for transient ones; which one we take is
  decided on a prototype by looks and load (section 5 of `ARCHITECTURE.md`). Custom blur shaders and
  constant repainting for the sake of the effect are forbidden. On systems without material support the
  window runs on fallback colors with no loss of function.

## 7. Icons

The only source is the **Hugeicons** set (hugeicons.com), one style across the whole application
(Stroke Rounded with a 1.5 px line is recommended). Styles must not be mixed and icons from other sets
must not be blended in.

- Icons are stored as a **font subset** (only the glyphs in use) and referenced through `enum Icon` in
  `ui/icons.rs`. An icon's color is a text or status token.
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
| Switch, checkbox | Off: `stroke.control` border; on: `accent` fill |
| Card | `surface.card`, radius 8, `stroke.subtle` border, no shadow |
| Navigation | On the left; the selected item gets `accent.subtle` and a 3 px vertical `accent` bar |
| Dialog | `surface.overlay`, radius 8, background dimming with `surface.sunken` at 60% opacity |
| Status badge | Icon + text, `status.*` color, `surface.sunken` background |
| Tooltip | Appears after ~500 ms, `surface.overlay`, Caption |

Every state changes smoothly under the rules of section 9; there are no "instant" switches.

### Application shell and brand neutrality

The application does not belong to a single manufacturer, so the shell is built around the device, not the
brand:
- **Navigation on the left:** "Devices" (the list of everything found), "Diagnostics", "Settings"; an
  opened device gets its own page.
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
| `spring.release` | 0.32 | 0.80 | ≈ 0.3 s, overshoot ≈ 1.5% | Sliders, the color-picker marker, return after a gesture |

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
| Press | Scale to 0.97 and `*.pressed` color, `spring.fast`; release returns taking velocity into account |
| Keyboard focus | The `focus.ring` fades in (opacity 0 → 1), `spring.fast` |
| Switch, checkbox | The thumb/check travels to the new position, color flows, `spring.standard`; the thumb stretches slightly while pressed |
| Navigation item selection | The 3 px `accent` bar **travels** from the old item to the new one instead of fading out and flashing; `spring.standard` |
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
| Going deeper (open details, a report) | Drill In (`Drill In`) | New page: scale 0.96 → 1 and opacity; the old one leaves fading |
| Going back | Reverse drill in | Mirrors the forward one |
| Window/dialog | Scale + opacity | Section 9.3 |

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
- [ ] No gradients, emoji, `#FFFFFF`/`#000000` (except text), or 255/0 channels in styling.
- [ ] Spacing and sizes are multiples of 4; radii are 4 or 8.
- [ ] Icons only from Hugeicons, one style, through `enum Icon`.
- [ ] Contrast recomputed if the palette changed.
- [ ] The window backdrop is visible only in the navigation area, and an opaque fallback exists.
- [ ] Every new action and state change has feedback under section 9, with values only from `spring.*` tokens.
- [ ] An animation can be interrupted at any moment without a jump; input is not blocked during animation.
- [ ] With Windows animations turned off, the interface remains usable.
- [ ] The idle window does not repaint (checked in Task Manager), including after animations.
- [ ] A before/after screenshot is attached to the change description.
