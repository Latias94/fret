# Material 3 Picker Packet v1

Date: 2026-05-27
Task: M3CAS-060
Components: DatePicker, TimePicker

## Scope

This packet audited the picker family after field and overlay foundations were stable. The focus was
not a full upstream port. It classified whether picker drift was caused by Material recipe code,
shared Material foundation code, `fret-ui-kit` overlay/focus policy, diagnostics, or stale headless
goldens.

## Reference Baseline

- Compose Material 3:
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/DatePicker.kt`
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/DatePickerDialog.kt`
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TimePicker.kt`
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TimePickerDialog.kt`
- Token baseline:
  - `repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-date-picker-docked.scss`
  - `repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-date-picker-modal.scss`
  - `repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-time-picker.scss`
  - `repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-time-input.scss`

## Findings

| Area | Classification | Result |
| --- | --- | --- |
| Modal overlay, scrim, focus trap/restore | `kit_policy` | Existing overlay controller and focus-scope primitives are the right boundary. No new mechanism change was needed. |
| Date month navigation and day grid | `material_recipe` | Existing staged month/selected-date model is recipe-owned and remains local to DatePicker. |
| Time dial/input display modes | `material_recipe` | Existing staged time model, selector keyboard handling, dial pointer handling, and input auto-advance are recipe-owned. |
| Stable automation selectors | `diagnostics` + `material_recipe` | Old hyphen/global ids were replaced with base-derived dotted part ids. |
| Headless picker golden drift | `test_harness` | The current scenes are stable. Previous picker goldens encoded stale stretched underlay/action-button geometry. |
| Accessibility parity depth | `follow_on` | Richer calendar/time-grid semantics should be split from this selector/golden packet. |

## Implemented Contract

DatePicker now derives stable ids from the supplied base:

- `date_picker`
- `date_picker.chrome`
- `date_picker.docked.prev`
- `date_picker.docked.next`
- `date_picker.modal.prev`
- `date_picker.modal.next`
- `date_picker.cell.<row>.<col>`
- `date_picker.cell.<yyyy-mm-dd>`
- `date_picker.scrim`
- `date_picker.scrim.chrome`
- `date_picker.panel`
- `date_picker.actions.cancel`
- `date_picker.actions.confirm`

TimePicker now derives stable ids from the supplied base:

- `time_picker`
- `time_picker.chrome`
- `time_picker.mode-toggle`
- `time_picker.hour-selector`
- `time_picker.hour-selector.chrome`
- `time_picker.minute-selector`
- `time_picker.minute-selector.chrome`
- `time_picker.clock-dial`
- `time_picker.clock-dial.chrome`
- `time_picker.clock-dial.hour.<HH>`
- `time_picker.clock-dial.minute.<MM>`
- `time_picker.period.am`
- `time_picker.period.pm`
- `time_picker.input.hour`
- `time_picker.input.hour.chrome`
- `time_picker.input.minute`
- `time_picker.input.minute.chrome`
- `time_picker.input.period.am`
- `time_picker.input.period.pm`
- `time_picker.scrim`
- `time_picker.scrim.chrome`
- `time_picker.panel`
- `time_picker.actions.cancel`
- `time_picker.actions.confirm`

The UI gallery TimePicker chrome-fill diagnostic was updated to use these base-derived selectors.

## Evidence

- `ecosystem/fret-ui-material3/src/date_picker.rs`
- `ecosystem/fret-ui-material3/src/time_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `docs/workstreams/material3-date-picker-day-cell-selectors-packet-v1/artifacts/date_picker_day_cell_selectors_packet_v1.md`
- `docs/workstreams/material3-time-picker-dial-accessibility-packet-v1/artifacts/time_picker_dial_accessibility_packet_v1.md`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-time-picker-chrome-fill.json`
- `goldens/material3-headless/v1/material3-date-picker.*.json`
- `goldens/material3-headless/v1/material3-time-picker.*.json`

## Gates

```powershell
cargo fmt --package fret-ui-material3
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_clock_dial_drag_updates_time
cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_selector_keyboard_arrows_step_time
cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_time_input_replaces_and_auto_advances_hour
$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_date_picker_suite_goldens_v1; Remove-Item Env:FRET_UPDATE_GOLDENS
$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_time_picker_suite_goldens_v1; Remove-Item Env:FRET_UPDATE_GOLDENS
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_date_picker_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_time_picker_suite_goldens_v1
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/WORKSTREAM.json > $null
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null
python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-time-picker-chrome-fill.json > $null
python tools/check_workstream_catalog.py
```

## Residual Risk

- DatePicker does not yet expose a full Material `SelectableDates`/disabled-date policy or
  localized day/month announcement surface.
- TimePicker does not yet expose invalid-input supporting/error text or richer live-region
  announcements.
- These are accessibility depth follow-ons, not blockers for the current component/foundation
  classification. They should be split if M3CAS-070/M3CAS-080 shows a shared a11y primitive need.
