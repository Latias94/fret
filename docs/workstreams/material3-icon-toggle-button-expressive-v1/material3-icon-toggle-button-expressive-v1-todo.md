# Material 3 Expressive Icon Toggle Button (v1) — TODO

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Compose Multiplatform (Material3):
  - `IconToggleButton` + expressive shapes: `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/IconButton.kt`
- Material UI:
  - `ToggleButton`: `repo-ref/material-ui/packages/mui-material/src/ToggleButton/ToggleButton.js`
  - `ToggleButtonGroup`: `repo-ref/material-ui/packages/mui-material/src/ToggleButtonGroup/ToggleButtonGroup.js`

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Milestones: `docs/workstreams/material3-icon-toggle-button-expressive-v1/material3-icon-toggle-button-expressive-v1-milestones.md`
Plan: `docs/workstreams/material3-icon-toggle-button-expressive-v1/material3-icon-toggle-button-expressive-v1-refactor-plan.md`

## TODO (ordered)

### A) Component surface (ecosystem/fret-ui-material3)

- [x] Decide API shape:
  - (Preferred) `IconToggleButton::new(Model<bool>, IconId)` (Fret-native),
  - or Compose-like `checked + on_checked_change`, with a `Model<bool>` adapter.
- [x] Implement toggle event wiring:
  - pointer activation toggles checked (or calls `on_checked_change`),
  - keyboard activation parity (Space/Enter) matches other pressables.
- [ ] Add a `toggle_style` surface (if needed) that can override:
  - container/background, icon color, outline color, state layer color.

### B) Expressive shapes (ecosystem/fret-ui-material3)

- [x] Add an `IconToggleButtonShapes` equivalent:
  - unchecked, pressed, checked corner radii (v1 = corners-only).
- [x] Implement morph selection rule: `pressed > checked > unchecked`.
- [x] Add checked-shape tokens (or a derived fallback):
  - e.g. `md.comp.icon-button.checked.container.shape` (final key depends on token import policy),
  - fall back to base shape if missing.
- [x] Ensure animations are stable:
  - scene structure remains stable during press and during checked transitions,
  - geometry stabilizes after settle (similar to existing icon button stability gates).

### C) A11y semantics decision (cross-platform)

- [x] Choose one strategy and document it in the plan:
  - Compose-aligned: `role=Checkbox` + `checked`,
  - Web/MUI-aligned: `role=Button` + `selected` (aria-pressed equivalent).
- [x] Remove the “both selected + checked” ambiguity in the final implementation.
- [x] Add a Rust/headless semantics snapshot test that asserts:
  - role, label, and checked/selected flags.

### D) UI gallery surface (apps/fret-ui-gallery)

- [x] Add an interactive demo section to the Material3 Icon Button page:
  - one `IconToggleButton` backed by a `Model<bool>`,
  - plus explicit “checked marker” text for easy diag assertions.
- [x] Add stable `test_id` for:
  - the toggle button,
  - the checked marker,
  - (optional) the Expressive variant switch row.

### E) Diagnostics + gates

- [x] Add a baseline diag capture script (screenshots + bundle):
  - `tools/diag-scripts/ui-gallery-material3-icon-toggle-button-expressive-screenshots.json`
- [x] Update the script to use `test_id` selectors once the gallery adds them.
- [x] Add a “behavioral” diag gate after the toggle becomes interactive:
  - click toggles checked marker (and/or semantics checked changes),
  - optional: pixel-change checks for checked/unchecked visuals.

## Evidence anchors (fill as tasks land)

- Component implementation: `ecosystem/fret-ui-material3/src/icon_button.rs`
- Token helpers: `ecosystem/fret-ui-material3/src/tokens/icon_button.rs`
- Expressive context plumbing: `ecosystem/fret-ui-material3/src/foundation/context.rs`
- Gallery surface: `apps/fret-ui-gallery/src/ui/previews/material3/controls.rs`
- Diagnostics script: `tools/diag-scripts/ui-gallery-material3-icon-toggle-button-expressive-screenshots.json`
- Headless semantics test: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`icon_toggle_button_semantics_role_and_checked_state_are_stable`)
- Headless animation stability tests:
  - `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`icon_button_pressed_scene_structure_is_stable`)
  - `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`icon_toggle_button_checked_transition_scene_structure_is_stable`)
