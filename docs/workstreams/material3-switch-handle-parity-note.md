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

### 1) No thumb content / icon support

Material Web and Compose both support icons/content inside the thumb:

- Material Web: `icons` + `show-only-selected-icon` and a `.handle.with-icon` sizing variant.
- Compose: `thumbContent` (and thumb size chooses the larger diameter when content exists).

Fret switch currently renders an empty thumb container and does not expose a policy surface to
provide thumb content or show-only-selected icon behavior.

**Practical impact**

- We cannot match Material Web “icons” demos or token behavior (`md.comp.switch.*.icon.*`).
- Handle sizing cannot follow the `with-icon` token path (`md.comp.switch.with-icon.handle.*`).

### 2) With-icon handle sizing is not modeled

Material Web has an explicit handle size variant when icons are present:

- `md.comp.switch.with-icon.handle.width`
- `md.comp.switch.with-icon.handle.height`

Fret currently only models:

- selected handle size,
- unselected handle size,
- pressed handle size.

This means even if we add icons later, the handle geometry will remain incorrect until we thread
the “has thumb content” state into the geometry rules.

### 3) Ripple/state-layer geometry needs explicit evidence

Upstream behavior uses a state layer that is not the same as the track bounds. Fret currently
implements a dedicated `ink_bounds` square around the handle center and drives ripple radius via
`md.comp.switch.state-layer.size`.

We should validate:

- ripple clip (bounded vs unbounded) matches our intended upstream reference (Material Web vs Compose),
- the ripple does not drift/jump under pressed handle resizing,
- state layer remains centered on the handle across transitions.

## Evidence + gates

- Baseline diag capture script (screenshots + bundle):
  - `tools/diag-scripts/ui-gallery-material3-switch-handle-screenshots.json`

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

