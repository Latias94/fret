# Material 3 Switch handle parity note (Material Web + Compose)

## Why this note exists

Switch “handle” visuals are easy to drift during refactors because they are the combination of:

- handle size changes (unchecked vs checked vs pressed),
- handle position rules (including pressed edge compensation),
- optional thumb icons and the “with-icon” handle size variant,
- state layer + ripple geometry (state layer is not the same as track bounds),
- disabled opacity rules that interact with track colors.

This note captures upstream outcomes and the current Fret implementation gaps so we can align
incrementally with evidence and gates.

## Upstream references (non-normative)

- Material Web Switch:
  - DOM + animation rationale: `repo-ref/material-web/switch/internal/README.md`
  - Handle sizing + icons + ripple container: `repo-ref/material-web/switch/internal/switch.ts`
  - Handle SCSS (sizes + pressed + with-icon): `repo-ref/material-web/switch/internal/_handle.scss`
- Compose Material3 Switch:
  - Thumb sizing + offset rules: `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Switch.kt`

## Current Fret implementation anchors

- Component: `ecosystem/fret-ui-material3/src/switch.rs`
- Tokens: `ecosystem/fret-ui-material3/src/tokens/switch.rs`
- Token source (v30): `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (prefix `md.comp.switch.*`)
- UI gallery surface: `apps/fret-ui-gallery/src/ui/previews/material3/controls.rs` (`preview_material3_switch`)

## Observed gaps / likely drift sources

### 1) Thumb icons: motion/visibility parity is not locked

Material Web and Compose both support icons/content inside the thumb:

- Material Web: `icons` + `show-only-selected-icon` and a `.handle.with-icon` sizing variant.
- Compose: `thumbContent` (and thumb size chooses the larger diameter when content exists).

Fret already supports `icons` and `show_only_selected_icon` at the recipe layer (see anchors below),
but we have not yet locked the **transition outcomes** (e.g. icon fade/rotate details) against the
upstream references.

**Practical impact**

- Visual drift is most likely to show up during state transitions (unchecked ↔ checked, pressed ↔ released),
  especially in `show-only-selected-icon` mode where Material Web keeps the "on" icon in the DOM to animate it
  even when unchecked.

### 2) With-icon handle sizing parity needs evidence

Material Web has an explicit handle size variant when icons are present:

- `md.comp.switch.with-icon.handle.width`
- `md.comp.switch.with-icon.handle.height`

Fret models this variant and threads it into geometry, but we should keep a small gate that prevents
future refactors from regressing the `with-icon` sizing rules.

### 3) Ripple/state-layer geometry needs explicit evidence

Upstream behavior uses a state layer that is not the same as the track bounds. Fret currently
implements a dedicated `ink_bounds` square around the handle center and drives ripple radius via
`md.comp.switch.state-layer.size`.

We should validate:

- ripple clip (bounded vs unbounded) matches our intended upstream reference (Material Web vs Compose),
- the ripple does not drift/jump under pressed handle resizing,
- state layer remains centered on the handle across transitions.

### 4) Focus chroming is split (focus-within vs focus-visible)

Material Web uses mixed focus selectors for switch chroming:

- Handle + icons use `:focus-within` (mouse focus can tint the handle).
- Track chroming differs by state:
  - selected: `:focus-within`
  - unselected: `:focus-visible` (keyboard focus only)

If we treat “focused” as strictly `focus-visible` everywhere, we will miss the handle/icon focus tint
when the switch is mouse-focused. If we treat “focused” as “any focus” everywhere, we will force
unselected track chroming to appear on mouse-focus, which diverges from the reference behavior.

Current Fret intent:

- Keep the focus ring gated on `focus-visible` (accessibility + keyboard navigation).
- Split switch token-driven chroming so the handle/icons can respond to “any focus” without forcing
  unselected track focus chroming.

### 5) Thumb motion details (position + size)

Material Web switch uses distinct motion channels for the thumb:

- **Position**: handle-container margin transition (300ms overshoot cubic-bezier).
- **Size**: handle width/height transition (250ms, `easing-standard`), with a pressed override:
  - pressed down: 100ms linear
  - release: back to 250ms `easing-standard`

Fret aligns these as:

- Position: tween (unclamped easing) to preserve overshoot behavior.
- Size + pressed: tweens matching the Material Web durations/easing.

## Evidence + gates

- Baseline diag capture script (screenshots + bundle):
  - `tools/diag-scripts/ui-gallery-material3-switch-handle-screenshots.json`
- Crossfade timeline evidence (captures frame-by-frame switch chrome transition):
  - `tools/diag-scripts/ui-gallery-material3-switch-chrome-crossfade-timeline-screenshots.json`
- Focus chroming evidence (click-focus vs focus-visible split):
  - `tools/diag-scripts/ui-gallery-material3-switch-focus-chroming-screenshots.json`
- Focus-visible evidence (keyboard modality flips focus ring + chroming selectors):
  - `tools/diag-scripts/ui-gallery-material3-switch-focus-visible-screenshots.json`
- Handle overshoot timeline evidence (captures frame-by-frame handle position transition):
  - `tools/diag-scripts/ui-gallery-material3-switch-handle-overshoot-timeline-screenshots.json`
- Icon motion timeline evidence (opacity + rotation timing):
  - `tools/diag-scripts/ui-gallery-material3-switch-icon-motion-timeline-screenshots.json`

Next gates (once we add icon support):

- Add a UI gallery demo that toggles `icons` / `show-only-selected-icon` (or an equivalent Fret surface).
- Add a diag script that captures:
  - unchecked (no icons),
  - checked (selected icon),
  - both-icons mode,
  - pressed state for each mode.
- Add a headless test that asserts handle geometry matches token expectations for:
  - unchecked/checked/pressed,
  - with-icon size overrides.
