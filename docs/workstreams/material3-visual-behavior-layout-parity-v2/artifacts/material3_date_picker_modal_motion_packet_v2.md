# Material3 DatePicker Modal Motion Packet v2

Date: 2026-05-28
Task: M3PV2-042

## Truth

- DatePickerDialog is a Material modal picker surface. Its modal presentation must fade the scrim
  and panel while the panel rises and scales into the centered final geometry.
- DatePicker modal motion should not drift from the shared Material Dialog modal choreography.
- The current DatePicker surface does not expose Compose's display-mode or year-picker transition
  APIs, so this packet covers the implemented docked/modal DatePicker motion surface.

## Sources

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/DatePickerDialog.kt`
  - DatePickerDialog is a Material dialog surface for DatePicker content.
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/androidMain/kotlin/androidx/compose/material3/DatePickerDialog.android.kt`
  - The implementation delegates modal behavior to `BasicAlertDialog` and wraps content in a
    `Surface` with DatePicker modal shape, width, height, color, and tonal elevation.
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/DatePicker.kt`
  - DatePicker's optional display-mode animation uses Default/Fast effects plus Default spatial
    slide/size motion.
  - Year picker visibility animates expand/shrink plus fade.
- Fret Material Dialog is the local modal-motion exemplar for implemented dialog surfaces.

## Artifacts

- `ecosystem/fret-ui-material3/src/foundation/modal_motion.rs`
- `ecosystem/fret-ui-material3/src/dialog.rs`
- `ecosystem/fret-ui-material3/src/date_picker.rs`
- `ecosystem/fret-ui-material3/tests/date_picker_motion.rs`

## Wiring

- Added `foundation::modal_motion::material_modal_panel_transform` so Material modal surfaces share
  the same fade/rise/scale panel transform instead of duplicating component-local math.
- Dialog now calls the shared helper with no intended visual behavior change.
- DatePickerDialog now uses the shared helper instead of its previous pure `0.95 -> 1.0` scale.

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --test date_picker_motion
```

It failed because the DatePicker modal panel faded but did not rise on the first open frame.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --test date_picker_motion
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_exposes_stable_part_test_ids material3_date_picker_month_label_is_polite_live_region material3_date_picker_uses_material_string_registry_and_date_descriptions material3_date_picker_respects_selectable_dates
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_date_picker_suite_goldens_v1 material3_headless_menu_dialog_style_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --lib date_picker
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json
python tools/check_workstream_catalog.py
git diff --check
```

## Matrix Impact

- `date_picker.motion`: `covered_v2`.

## Residual Risk

- Future DatePicker display-mode and year-picker UI should wire Compose-style spatial/effects
  transitions when those surfaces exist in Fret. They are not part of the current public recipe.
- TimePickerDialog still uses its own modal panel motion and should move to the shared helper in a
  follow-on TimePicker motion packet.
