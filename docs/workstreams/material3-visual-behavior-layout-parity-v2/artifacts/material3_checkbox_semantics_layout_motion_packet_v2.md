# Material3 Checkbox Semantics Layout Motion Packet v2

Date: 2026-05-29
Task: M3PV2-065

## Truth

- Checkbox exposes `Checkbox` semantics with explicit binary and tri-state checked metadata.
- Indeterminate checkboxes expose `checked_state = Mixed`, while the legacy binary checked flag
  remains absent for mixed state.
- Checkbox keeps Material's 48px minimum interactive target, 40px state-layer chrome, and 18px
  visual box/mark geometry.
- Checkbox publishes stable `.chrome`, `.box`, and `.mark` part ids for automation.
- The checked mark does not snap directly to settled visibility after toggle; it advances over
  fixed frames using the Material `DefaultSpatial` motion scheme.

## Sources

- Compose Material3 `Checkbox.kt`: `TriStateCheckbox` uses `triStateToggleable(... role =
  Role.Checkbox)`, `minimumInteractiveComponentSize()`, unbounded ripple with radius
  `CheckboxTokens.StateLayerSize / 2`, and `CheckboxImpl` animates selected mark draw fraction with
  `MotionSchemeKeyTokens.DefaultSpatial`.
- Compose Material3 `CheckboxTokens.kt`: container size `18dp`, icon size `18dp`, state-layer size
  `40dp`, selected/unselected/disabled color and outline outcomes.
- Fret `fret-ui-kit::primitives::checkbox::checkbox_a11y`: already maps
  `CheckedState::Indeterminate` to `SemanticsCheckedState::Mixed`.
- Existing Fret Material indication foundation already had Material state-layer/ripple timing and
  `SpringAnimator`/motion-scheme helpers.

MUI Material UI is not available in this worktree's `repo-ref/`; this packet used local Compose,
generated Material Web v30 token snapshots, and existing Fret kit primitive semantics.

## Layer Finding

This packet found Material recipe wiring gaps plus a reusable Material foundation capability that
already existed:

- `fret-core` already exposes `SemanticsCheckedState::Mixed`; no core semantics contract change was
  required.
- `fret-ui-kit` already has the Radix-style checkbox a11y helper that writes both legacy binary
  `checked` and tri-state `checked_state`.
- `Checkbox` was bypassing that helper and only wrote `checked: Option<bool>`, so mixed state had no
  explicit checked-state metadata.
- `Checkbox` exposed `.chrome` but not the 18px box/mark parts, making Material geometry harder to
  prove at shadcn-level automation density.
- `Checkbox` used instant SVG mark visibility. The fix reuses existing Material motion-scheme
  springs instead of adding new core animation primitives.

No cross-crate mechanism change was required.

## Artifacts

- `ecosystem/fret-ui-material3/src/checkbox.rs`
- `ecosystem/fret-ui-material3/tests/checkbox_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test checkbox_state
```

The new tests initially failed because Checkbox did not expose `.box` / `.mark` test ids, did not
write `checked_state`, and emitted no animated mark opacity after toggle.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test checkbox_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_choice_controls_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment checkbox_tristate_semantics_and_toggle_outcomes checkbox_pressed_scene_structure_is_stable material3_headless_controls_suite_goldens_v1
python -m json.tool docs\workstreams\material3-visual-behavior-layout-parity-v2\WORKSTREAM.json | Out-Null
python -m json.tool docs\workstreams\material3-visual-behavior-layout-parity-v2\artifacts\material3_parity_axis_matrix_v2.json | Out-Null
python tools\check_workstream_catalog.py
git diff --check
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

## Residual Risk

- Checkbox still uses SVG glyphs for the check/minus mark rather than a path-draw fraction that
  exactly matches Compose's check geometry.
- Error-state checkbox colors are not exposed because the current public Checkbox recipe has no
  `error(bool)` surface; this should be revisited only if Fret chooses to model Material Web's
  checkbox error variant.
- Radio, Switch, Slider, SegmentedButton, and chip families still need their own choice-control
  packets.
