# Material3 Select Trigger Motion Packet v2

Task: M3PV2-036
Date: 2026-05-28
Status: Complete

## Truth

Material Select trigger chrome is field-like. Its label, placeholder, outline, and filled active
indicator should use the same Material field transition model as TextField:

- an initially populated Select mounts with the label already floated;
- focus/open/populated state changes animate the floating label through intermediate fixed-clock
  frames instead of snapping or delaying at the idle geometry;
- outlined border and filled active-indicator targets are driven by the shared field motion policy;
- label automation has the same dotted part-id shape as TextField (`<base>.label`).

## Sources

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/internal/TextFieldImpl.kt`
  - Field label progress is driven by `TextFieldTransitionScope` and Material motion tokens.
- `repo-ref/material-ui/packages/mui-material/src/Select/Select.js`
  - Select composes input-field chrome rather than owning an unrelated field transition model.
- Fret Material3 TextField M3PV2-035:
  - `field_motion_frame` behavior was already proven for first-frame label motion and active
    indicator thickness in TextField.

## Findings

This was a Material recipe/foundation divergence, not a core `fret-ui` mechanism issue.

Select had a component-local `StateLayerAnimator` for floating-label progress while TextField used
Material spring motion. That created two observable drifts:

- initially selected Selects started from idle label geometry (`first=19`, `settled=7`) instead of
  mounting at the populated/floated state;
- empty Selects gained focus with field chrome already changing while label progress had not
  advanced (`idle=19`, `first=21` in the red gate), so the first visible frame was not the same
  TextField-like transition.

The root issue was duplicated field motion logic. The fix was to lift TextField's motion runtime
into shared Material foundation code and use it from both TextField and Select.

## Changes

- Added `foundation::field_motion` with `FieldInputPhase`, `FieldMotionTargets`,
  `FieldMotionFrame`, and `field_motion_frame`.
- Rewired TextField to use the shared foundation helper without changing its public API.
- Rewired Select trigger label/placeholder/outline/active-indicator targets through the shared
  field motion helper.
- Added Select label part test id support (`<base>.label`) and updated the automation surface.
- Added fixed-frame Select trigger motion gates:
  - initial populated Select mounts label at settled floated geometry;
  - focus transition label first frame lands between idle and focused geometry.

## Gates

Red before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_initial_selected_label_mounts_at_settled_floating_position
cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_focus_floating_label_animates_between_idle_and_focused
```

Failed with:

```text
expected outlined initial selected Select label to mount at its settled floating position: first=19, settled=7, delta=12
expected outlined Select floating label to start moving on the first focus frame: idle=19, first=21
```

Green:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_initial_selected_label_mounts_at_settled_floating_position select_focus_floating_label_animates_between_idle_and_focused
cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_floating_label_animates_between_idle_and_focused filled_text_field_focus_uses_focus_indicator_thickness
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_select_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior
cargo nextest run -p fret-ui-material3 --test text_field_hover
cargo nextest run -p fret-ui-material3 --test select_behavior
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
git diff --check
```

## Residual Risk

- This packet closes Select trigger field-motion drift. It does not add a per-Select fixed-frame
  probe for chevron rotation.
- Select overlay enter/exit still uses the shared `foundation::overlay_motion` helper; a future
  overlay-family packet should probe alpha/scale at fixed frames across Select/Menu/Tooltip/Search.
