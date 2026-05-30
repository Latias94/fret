# Material3 TimePicker Modal Motion Packet v2

Date: 2026-05-28
Task: M3PV2-043

## Truth

- TimePickerDialog is a Material modal picker surface. Its modal presentation must fade the scrim
  and panel while the panel rises and scales into the centered final geometry.
- TimePickerDialog must not carry a component-local modal transform that drifts from Dialog and
  DatePickerDialog.
- This packet covers modal open/close motion for both initial dial and input display modes. It does
  not close the TimePicker clock-face selector/crossfade motion gaps.

## Sources

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TimePickerDialog.kt`
  - TimePickerDialog is a Material dialog surface hosting time picker content and actions.
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TimePicker.kt`
  - `ClockDialModifier` receives `MotionSchemeKeyTokens.DefaultSpatial` for selector movement.
  - `ClockFace` crossfades between hour/minute value sets with `MotionSchemeKeyTokens.DefaultEffects`.
- Fret M3PV2-042 established `foundation::modal_motion` as the shared Material modal panel
  transform for Dialog and DatePickerDialog.

## Artifacts

- `ecosystem/fret-ui-material3/src/time_picker.rs`
- `ecosystem/fret-ui-material3/tests/time_picker_motion.rs`

## Wiring

- TimePickerDialog now calls `foundation::modal_motion::material_modal_panel_transform` for its
  modal panel transform.
- The fixed-frame gate covers both `TimePickerDisplayMode::Dial` and `TimePickerDisplayMode::Input`
  as initial modal content.

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --test time_picker_motion
```

It failed because TimePickerDialog faded but did not rise on the first open frame.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --test time_picker_motion
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids material3_time_picker_uses_compose_aligned_accessibility_labels material3_time_picker_uses_material_string_registry
cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_clock_dial_drag_updates_time time_picker_selector_keyboard_arrows_step_time time_picker_time_input_replaces_and_auto_advances_hour time_picker_time_input_rejects_invalid_values_and_recovers material3_headless_time_picker_suite_goldens_v1
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json
python tools/check_workstream_catalog.py
git diff --check
```

## Matrix Impact

- `time_picker.motion` was still open after this packet.
- Later clock-face and 24h dial packets, plus the final closeout matrix, supersede that interim
  state.

## Residual Risk

- TimePicker clock-face selector movement still snaps instead of using Compose's DefaultSpatial
  selector motion.
- Hour/minute clock-face value changes still snap instead of crossfading with DefaultEffects.
