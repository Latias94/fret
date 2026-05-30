# Material3 TimePicker 24h Clock-Face Layout Packet v2

Date: 2026-05-29
Task: M3PV2-045

## Truth

- In 24-hour hour-selection mode, TimePicker must render `00..11` on the outer ring and `12..23`
  on the inner ring.
- Paired hours such as `01` and `13` must share the same clock angle with different radii.
- Pointer selection in 24-hour mode must use pointer distance from the dial center to distinguish
  outer-ring AM hours from inner-ring PM hours.
- The default 12-hour dial should keep the same 12-position semantics while adopting Compose's
  outer dial radius.

## Sources

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TimePicker.kt`
  - `ClockFace` renders the main hour values on the outer `CircularLayout`.
  - For 24-hour hour selection it renders `ExtraHours` on an inner `CircularLayout`.
  - `moveSelector` uses `MaxDistance` to choose inner vs outer hour ring from pointer distance.
  - Source constants define outer radius `101dp / 256dp`, inner radius `69dp / 256dp`, and split
    distance `74dp / 256dp`.

## Artifacts

- `ecosystem/fret-ui-material3/src/time_picker.rs`
- `ecosystem/fret-ui-material3/tests/time_picker_clock_face.rs`
- `goldens/material3-headless/v1/material3-time-picker.*.json`

## Wiring

- TimePicker dial labels now carry explicit `ring`, `angle_idx`, `label`, and committed `value`
  metadata.
- 24h hour labels are generated as two 12-position rings: outer `00..11`, inner `12..23`.
- Selector radius is now spring-driven along with selector angle so it can move between inner and
  outer rings.
- Pointer-driven dial selection maps hour taps through the same 12-position angle model and uses
  the Compose ring-split ratio for 24h AM/PM selection.

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --test time_picker_clock_face
```

It failed because `01` and `13` were opposite angles on a single 24-item ring, and pressing the
intended inner-ring `13` position selected `02`.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --test time_picker_clock_face
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

- `time_picker.layout` stays `covered_v2`, now with the 24h inner/outer ring gap closed.
- `time_picker.first_v2_gate` now explicitly names the 24h clock-face ring packet.

## Residual Risk

- TimePicker style remains `covered_v1` in the matrix because a full token/state matrix for every
  display/input/dial subpart has not yet been run as a v2 packet.
