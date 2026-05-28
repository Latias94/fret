# Material3 TimePicker Clock-Face Motion Packet v2

Date: 2026-05-28
Task: M3PV2-044

## Truth

- TimePicker's analog clock face must not snap when selecting an hour and auto-switching to the
  minute face.
- The selector handle must move with Material spatial motion along the dial instead of replacing
  the selected label background in place.
- Hour/minute value sets must crossfade during the selection-mode change.

## Sources

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TimePicker.kt`
  - `ClockDialModifier` uses `MotionSchemeKeyTokens.DefaultSpatial` and calls
    `state.animateToCurrent(...)` when the selection mode changes.
  - `ClockFace` wraps value sets in `Crossfade(... MotionSchemeKeyTokens.DefaultEffects.value())`.
  - `drawSelector` draws an independent selector handle, track, and center dot.
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/TimePickerTokens.kt`
  - Defines selector handle, center, and track token dimensions/colors.

## Artifacts

- `ecosystem/fret-ui-material3/src/time_picker.rs`
- `ecosystem/fret-ui-material3/src/tokens/time_picker.rs`
- `ecosystem/fret-ui-material3/tests/time_picker_motion.rs`
- `goldens/material3-headless/v1/material3-time-picker.*.json`

## Wiring

- TimePicker clock dial now keeps a component-local motion runtime for:
  - an angle spring driven by Material `DefaultSpatial`,
  - a face-alpha spring driven by Material `DefaultEffects`,
  - outgoing face retention while hour/minute labels crossfade.
- The selected label background was replaced by independent selector chrome: track, center dot, and
  handle. This keeps the label pressable hitboxes stable while the visual selector moves.
- TimePicker token access now exposes the selector center and track tokens already present in the
  Material Web v30 token table.

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --test time_picker_motion docked_time_picker_clock_face_crossfades_and_moves_selector_on_selection_change
```

It failed because the first frame after hour selection had no intermediate opacity and no selector
translation.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --test time_picker_motion
$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_time_picker_suite_goldens_v1; Remove-Item Env:\FRET_UPDATE_GOLDENS
cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_clock_dial_drag_updates_time time_picker_selector_keyboard_arrows_step_time time_picker_time_input_replaces_and_auto_advances_hour time_picker_time_input_rejects_invalid_values_and_recovers material3_headless_time_picker_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids material3_time_picker_uses_compose_aligned_accessibility_labels material3_time_picker_uses_material_string_registry
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json
python tools/check_workstream_catalog.py
git diff --check
```

## Matrix Impact

- `time_picker.motion` is now `covered_v2`.

## Residual Risk

- This packet covers 12-hour dial mode. The existing Fret implementation still represents the
  24-hour clock face as a single 24-item ring rather than Compose's inner/outer hour rings; that is
  a future layout/style parity packet, not a blocker for the current selector/crossfade motion
  closure.
