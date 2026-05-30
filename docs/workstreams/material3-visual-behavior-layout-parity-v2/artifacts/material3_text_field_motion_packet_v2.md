# Material3 TextField Motion Packet v2

Task: M3PV2-035
Date: 2026-05-28
Status: Complete

## Truth

Material TextField motion is part of the component recipe. Focus changes should not only resolve to
the correct settled label and active-indicator geometry; intermediate fixed-clock frames must show
movement from the previous state to the target state.

The expected observable outcomes are:

- Floating label progress uses the Material fast spatial motion spec.
- First focus frame lands between idle and focused label geometry, not at either endpoint.
- Filled active indicator thickness animates from the idle `1px` line to the focused `2px` line.
- Single-line and multiline TextFields share the same motion behavior.

## Sources

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/internal/TextFieldImpl.kt`
  - `TextFieldTransitionScope` uses `updateTransition(inputState)`.
  - `labelProgress` uses `MotionSchemeKeyTokens.FastSpatial`.
  - Placeholder opacity switches between fast and slow effects based on input-phase transitions.
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TextField.kt`
  - Active indicator width animation uses spatial motion; indicator color uses effects motion.
- Fret Material3:
  - `foundation::motion_scheme::sys_spring_in_scope` resolves Material motion spring tokens.
  - `motion::SpringAnimator` is deterministic by `FrameId`, which makes fixed-frame tests viable.

## Findings

The bug was in the Material TextField recipe, not in `fret-ui` core.

`TextFieldRuntime` initialized border and placeholder animators during the idle frame because those
paths always called `set_target`. The floating-label animator only called `set_target` when
`float_target` changed. Since the runtime default target was also `false`, the first idle render did
not initialize the label spring. On the first focus render, `SpringAnimator::set_target` saw an
uninitialized animator and reset directly to `1.0`, so the label snapped to the focused endpoint.

The multiline branch had a parallel gap: it resolved `float_progress` and placeholder opacity
immediately instead of sharing the animated TextField runtime.

## Changes

- Added a red fixed-frame gate proving the first focus frame must sit between idle and focused label
  y positions.
- Added first-frame active-indicator thickness proof to the existing filled focus test.
- Extracted `TextFieldMotionTargets`, `TextFieldMotionFrame`, and `text_field_motion_frame` so
  single-line and multiline branches share the same spring state and phase logic.
- Initialized the floating-label spring on the idle frame, while still resetting immediately for
  disabled TextFields.
- Reused the shared motion frame for multiline TextArea-backed TextFields.

## Gates

Red before fix:

```powershell
cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_floating_label_animates_between_idle_and_focused
```

Failed with:

```text
expected outlined floating label to animate instead of snapping to the focused endpoint: first=6, settled=6
```

Green:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_floating_label_animates_between_idle_and_focused filled_text_field_focus_uses_focus_indicator_thickness
cargo nextest run -p fret-ui-material3 --test text_field_hover
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
git diff --check
```

## Residual Risk

- Placeholder opacity now goes through the shared TextField motion frame, but the scene harness does
  not currently expose text alpha in a small assertion helper. Add a text-paint alpha signature if a
  future packet needs pixel-level placeholder fade proof.
- This packet closes TextField motion. Select, Autocomplete, ExposedDropdown, SearchBar/SearchView,
  DatePicker, and TimePicker still need their own fixed-frame motion packets.
