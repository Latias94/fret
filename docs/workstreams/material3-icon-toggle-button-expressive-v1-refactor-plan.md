# Material 3 Expressive Icon Toggle Button (v1) — Refactor plan

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Compose Multiplatform (Material3):
  - `IconToggleButton` + expressive shape morph: `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/IconButton.kt`
- Material UI (web, React):
  - `ToggleButton` + selected styling: `repo-ref/material-ui/packages/mui-material/src/ToggleButton/ToggleButton.js`
  - `ToggleButtonGroup` border/rounding policy: `repo-ref/material-ui/packages/mui-material/src/ToggleButtonGroup/ToggleButtonGroup.js`

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

## Motivation

Fret already has a Material 3 `IconButton` MVP in `ecosystem/fret-ui-material3`, including:

- token-driven color selection (`md.comp.icon-button.*`),
- state layer + ripple (`foundation::indication`),
- focus-visible ring (`foundation::focus_ring`),
- a pressed-corner morph (pressed radius animation).

However, the **toggleable** icon button story is currently incomplete and not Expressive-aligned.
In Compose Material3, toggleable icon buttons are first-class (`IconToggleButton`, `FilledIconToggleButton`, ...),
and Expressive explicitly introduces **shape morphing across interaction + checked state** via `IconToggleButtonShapes`.

Goal: land a v1 “Icon Toggle Button” surface that:

- matches Compose Material3 outcomes (checked behavior, state layer/ripple, semantics),
- exposes a Fret-native controlled surface (model-driven, consistent with `Checkbox`/`Switch`),
- supports Expressive shape morph rules (unchecked/pressed/checked),
- is gateable via UI gallery + diag scripts.

## Current state (in-tree anchors)

- Component: `ecosystem/fret-ui-material3/src/icon_button.rs`
  - First-class toggle contract exists: `IconToggleButton::new(Model<bool>, IconId)`.
  - Legacy static toggle styling still exists via `IconButton.toggle(true)` + `IconButton.selected(bool)`,
    but `IconToggleButton` is the recommended surface for interactive behavior.
  - Expressive shape morph includes checked state (selection rule: `pressed > checked > unchecked`).
  - A11y is Compose-aligned for toggle mode: `role=Checkbox` + `checked`, and does not set `selected`.
- Tokens: `ecosystem/fret-ui-material3/src/tokens/icon_button.rs`
  - Provides base + pressed shape radius tokens and per-variant color keys.
  - Models a selected/checked shape radius token (Expressive expects checked shape to be distinct).
- Gallery surface: `apps/fret-ui-gallery/src/ui/previews/material3/controls.rs` (`preview_material3_icon_button`)
  - Contains an interactive `IconToggleButton` demo backed by a `Model<bool>`.
  - Stable `test_id` exists for both the toggle button and its checked marker.
- Existing stability gate: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`icon_button_pressed_scene_structure_is_stable`)
  - Validates pressed animation does not change scene structure.
  - Semantics gate exists for toggle button role + checked state:
    `icon_toggle_button_semantics_role_and_checked_state_are_stable`.

## Observed divergences vs Compose / MUI

Outcome-level gaps (these are not “just wrong tokens”):

1) **Missing toggle contract**
   - Compose: `checked: Boolean` + `onCheckedChange` (controlled).
   - Fret M3: `IconToggleButton::new(Model<bool>, IconId)` provides a controlled, interactive contract.
   - Result: gallery + diag script can gate toggle behavior deterministically.

2) **Expressive shape morph requires `checked`**
   - Compose Expressive: `IconToggleButtonShapes(shape, pressedShape, checkedShape)` and `shapeForInteraction(checked, ...)`.
   - Fret M3: expressive variant morphs across `pressed/checked/unchecked` corner radii.

3) **A11y role/flags are under-specified**
   - Compose sets `role = Role.Checkbox` for toggleable icon buttons.
   - MUI uses `aria-pressed` on a `button` (toggle button semantics).
   - Fret chooses Compose-aligned semantics for Material3 toggle icon buttons:
     `role=Checkbox` + `checked`, and does not set `selected`.
   - This is gated via a headless semantics test and a diag script predicate (`role_is` + `checked_is`).

4) **Selectors for diag gating are weak**
   - Stable `test_id` selectors are now present for gating.

## Refactor strategy (fearless, reversible)

### 1) Introduce a first-class `IconToggleButton` surface (policy stays in `fret-ui-material3`)

Add a new component type (or an `IconButton` constructor) that encodes the toggle contract:

- `IconToggleButton::new(checked: Model<bool>, icon: IconId)` (Fret-native, consistent with `Checkbox`/`Switch`), or
- `IconToggleButton::controlled(checked: bool).on_checked_change(...)` (Compose-like surface),
  with an adapter convenience for `Model<bool>`.

Key outcomes:

- Clicking toggles / updates checked state (or calls a callback) in a predictable way.
- The toggle mode no longer relies on “remember to set `.toggle(true)` and `.selected(...)` correctly”.

### 2) Model Expressive shapes explicitly

Introduce a `IconToggleButtonShapes`-like struct in `fret-ui-material3` that can drive:

- unchecked shape (base),
- pressed shape,
- checked shape,
- and a deterministic morph selection function: `pressed > checked > unchecked`.

Implementation note:

- Start with corner-radius based shapes (a `Corners`-only representation is fine for v1).
- Later, if we need arbitrary shapes, lift the shape system into `foundation` (but keep v1 narrow).

### 3) Decide and lock a11y semantics

Pick one of these strategies and gate it:

- **Compose-aligned**: `SemanticsRole::Checkbox` + `checked=Some(bool)` (preferred for platform parity),
  keep `selected=false`.
- **Web/MUI-aligned**: `SemanticsRole::Button` + `selected=true/false` (pressed) and do not set `checked`.

We should not set both `selected` and `checked` for the same control unless we have a strong cross-platform reason.

### 4) Add a UI gallery “interactive” demo + stable selectors

Update the Material3 icon button page to include:

- at least one interactive toggle icon button backed by a `Model<bool>`,
- stable `test_id`s for:
  - the toggle button itself,
  - the current checked state marker (text),
  - and (optionally) the Expressive variant switch.

### 5) Add diagnostics gates

Ship a minimal 3-pack:

- Repro script (diag): `tools/diag-scripts/ui-gallery-material3-icon-toggle-button-expressive-screenshots.json`
  - captures Standard + Expressive states, and clicks the toggle examples.
- Headless/Rust gate:
  - semantics snapshot asserts role + checked/selected strategy,
  - shape morph selection rules (pressed vs checked) are stable and don’t flicker scene structure.
- Evidence anchors added to the workstream TODO.

## Definition of done (v1)

- A first-class `IconToggleButton` surface exists in `ecosystem/fret-ui-material3`.
- Toggle behavior is interactive and controlled (model or callback), not “static examples”.
- Expressive shape morph is implemented (unchecked/pressed/checked).
- A11y strategy is explicitly chosen and regression-gated.
- UI gallery provides stable `test_id` selectors.
- At least one diag script produces a packed bundle with screenshots for review.
