# Material3 Switch Semantics Layout Motion Packet v2

Date: 2026-05-29
Task: M3PV2-067

## Truth

- Switch exposes `Switch` semantics with binary `checked` and explicit `checked_state`.
- Switch keeps Material's 48px minimum interactive target, 52px track width, 40px state-layer
  chrome, 32px track height, and selected 24px handle geometry.
- Switch publishes stable `.chrome`, `.track`, `.handle`, `.icon-on`, and `.icon-off` part ids.
- Toggling from off to on moves the handle toward the selected edge and grows/holds the handle over
  fixed frames rather than snapping without proof.
- Existing ripple/pressed scene gates remain green for pointer, keyboard, icon, and selected-only
  icon scenarios.

## Sources

- Compose Material3 `Switch.kt`: `Switch` applies `minimumInteractiveComponentSize()`,
  `toggleable(... role = Role.Switch)`, required 52x32dp visual size, unbounded thumb ripple with
  radius `SwitchTokens.StateLayerSize / 2`, and `MotionSchemeKeyTokens.FastSpatial` for thumb
  element offset/size.
- Compose Material3 `SwitchTokens.kt`: state-layer size `40dp`, track `52x32dp`, selected handle
  `24dp`, unselected handle `16dp`, pressed handle `28dp`, selected/unselected icon size `16dp`,
  and selected/unselected/disabled color outcomes.
- Fret `fret-ui-kit::primitives::switch::switch_a11y`: this packet promotes binary
  `checked_state` into the shared switch primitive so Material and IMUI users share the semantics
  fix.
- Existing Fret Material Switch implementation already had Material Web-compatible icon transition
  support, handle motion, ripple bounds, and stable part ids.

MUI Material UI is not mirrored in this worktree's `repo-ref/`; this packet used local Compose,
Base UI/Radix-style Fret kit primitive expectations, and the existing Material Web-aligned Fret
Switch implementation.

## Layer Finding

This packet found a shared kit primitive gap plus Material recipe wiring/proof-density gaps:

- `fret-core` already exposes `SemanticsCheckedState::{True,False}`; no core semantics contract
  change was required.
- `fret-ui-kit::primitives::switch::switch_a11y` only wrote the legacy binary `checked` flag.
  Updating the helper fixes Material Switch and any other design-system/user surface that reuses
  it.
- Material `Switch` bypassed the kit helper and manually wrote `role = Switch` plus `checked`.
- Material `Switch` already had 52/40/32/24px geometry, stable part ids, pointer/keyboard ripple
  gates, icon persistence gates, and handle motion behavior; the missing piece was a focused v2
  geometry/motion assertion and explicit checked-state semantics.

No core mechanism change was required.

## Artifacts

- `ecosystem/fret-ui-kit/src/primitives/switch.rs`
- `ecosystem/fret-ui-material3/src/switch.rs`
- `ecosystem/fret-ui-material3/tests/switch_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test switch_state
```

The new tests initially failed because Switch did not write explicit binary `checked_state`.
Geometry and handle-motion probes already passed, confirming the main implementation gap was
semantics wiring rather than layout or current motion behavior.

Green gates:

```powershell
cargo fmt --package fret-ui-kit --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test switch_state
cargo nextest run -p fret-ui-kit --lib switch_a11y_sets_role_and_checked switch_use_checked_model_prefers_controlled_and_does_not_call_default
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_switch_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment switch_ripple_origin_tracks_pointer_down_position switch_keyboard_ripple_origin_ignores_stale_pointer_down switch_ripple_holds_for_minimum_press_duration_before_fade switch_pressed_scene_structure_is_stable switch_icons_pressed_scene_structure_is_stable switch_selected_only_icon_persists_during_toggle_animation material3_headless_controls_suite_goldens_v1
python -m json.tool docs\workstreams\material3-visual-behavior-layout-parity-v2\WORKSTREAM.json | Out-Null
python -m json.tool docs\workstreams\material3-visual-behavior-layout-parity-v2\artifacts\material3_parity_axis_matrix_v2.json | Out-Null
python tools\check_workstream_catalog.py
git diff --check
cargo check -p fret-ui-kit --lib
cargo clippy -p fret-ui-kit --lib --no-deps -- -D warnings
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

## Residual Risk

- Fret Switch currently keeps its Material Web-compatible icon and handle transition details; this
  packet proves fixed-frame handle movement but does not replace that motion stack with Compose's
  exact `FastSpatial` thumb modifier.
- Drag/swipe gestures are still not implemented, matching the upstream Compose TODO in the local
  source.
- Slider, SegmentedButton, chips, and IconToggleButton still need their own choice-control packets.
