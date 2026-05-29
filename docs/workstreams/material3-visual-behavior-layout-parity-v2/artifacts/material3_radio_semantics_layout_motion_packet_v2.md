# Material3 Radio Semantics Layout Motion Packet v2

Date: 2026-05-29
Task: M3PV2-066

## Truth

- RadioGroup exposes `RadioGroup` semantics and each item exposes `RadioButton` semantics with
  binary `checked` and explicit `checked_state`.
- RadioGroup items keep collection metadata (`pos_in_set`, `set_size`) for assistive technologies.
- Radio keeps Material's 48px minimum interactive target, 40px state-layer chrome, 20px icon, and
  10px selected-dot geometry.
- Radio publishes stable `.chrome`, `.icon`, and `.dot` part ids for automation.
- Initially selected radios start visually settled, while a user selection transition grows the
  dot over fixed frames using the Material `FastSpatial` motion scheme.

## Sources

- Compose Material3 `RadioButton.kt`: `RadioButton` applies
  `minimumInteractiveComponentSize()`, `Modifier.selectable(... role = Role.RadioButton)`,
  unbounded ripple with radius `RadioButtonTokens.StateLayerSize / 2`, 2dp padding, a required
  20dp icon canvas, and `MotionSchemeKeyTokens.FastSpatial` for selected-dot radius animation.
- Compose Material3 `RadioButtonTokens.kt`: icon size `20dp`, state-layer size `40dp`, and
  selected/unselected/disabled icon colors.
- Base UI radio source: `RadioRoot` exposes `role="radio"` with checked state, and
  `Radio.Indicator` is a separate part whose checked/unchecked data state follows the root.
- Fret `fret-ui-kit::primitives::radio_group::radio_button_a11y`: already maps binary checked
  radio state to `SemanticsCheckedState::{True,False}`.

MUI Material UI is not mirrored in this worktree's `repo-ref/`; this packet used local Compose and
Base UI mirrors plus existing Fret kit primitive semantics.

## Layer Finding

This packet found Material recipe wiring and proof-density gaps, not a core or kit mechanism gap:

- `fret-core` already exposes explicit checked-state semantics.
- `fret-ui-kit` already has a radio a11y helper that writes both legacy binary `checked` and
  explicit `checked_state`.
- Material `Radio` bypassed that helper and only wrote the legacy binary `checked` flag.
- Material `Radio` already had roving/typeahead group behavior, collection metadata, a 48px touch
  target, a 40px state layer, a 20px icon, a 10px dot, and Material ripple/state-layer wiring.
- The missing recipe contracts were stable `.icon` / `.dot` diagnostic anchors and using Material
  motion-scheme springs for dot growth instead of duration/easing state-layer animation.

No cross-crate mechanism change was required.

## Artifacts

- `ecosystem/fret-ui-material3/src/radio.rs`
- `ecosystem/fret-ui-material3/tests/radio_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_state
```

The new tests initially failed because Radio did not expose `.icon` / `.dot` test ids, did not
write `checked_state`, and initially selected radios did not paint a settled dot on the first
frame.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_choice_controls_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment radio_selected_dot_is_centered_in_outline radio_ripple_origin_tracks_pointer_down_position radio_pressed_scene_structure_is_stable material3_headless_controls_suite_goldens_v1
python -m json.tool docs\workstreams\material3-visual-behavior-layout-parity-v2\WORKSTREAM.json | Out-Null
python -m json.tool docs\workstreams\material3-visual-behavior-layout-parity-v2\artifacts\material3_parity_axis_matrix_v2.json | Out-Null
python tools\check_workstream_catalog.py
git diff --check
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

## Residual Risk

- Radio color transitions still use the current token-state resolution path; this packet did not
  add a fixed-frame `DefaultEffects` color interpolation gate.
- Full form registration and native radio input parity from Base UI/MUI are outside the current
  Fret radio recipe surface.
- Switch, Slider, SegmentedButton, chips, and IconToggleButton still need their own choice-control
  packets.
