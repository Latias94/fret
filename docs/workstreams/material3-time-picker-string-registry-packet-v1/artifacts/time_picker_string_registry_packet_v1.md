# TimePicker String Registry Packet v1

## Truth

- TimePicker visible labels and automation-facing semantics no longer depend on recipe-local string
  constants.
- `foundation::strings` owns the Material string-key bridge and falls back to English Material
  outcomes when no `I18nService` backend/key is present.
- Docked dial, docked input, and modal dialog surfaces all consume the same Material string helpers.
- Bootstrap default i18n resources include TimePicker strings for `en-US` and `zh-CN`, including
  argument-backed spoken hour/minute values.

## Artifacts

- `ecosystem/fret-ui-material3/src/foundation/strings.rs`
- `ecosystem/fret-ui-material3/src/time_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Wiring

The registry uses `fret_runtime::fret_i18n::I18nService` from the host global store. Missing service
or missing keys return English fallback strings, so existing apps keep their previous behavior while
localized apps can override Material strings through the normal Fret i18n path.

## Proof

`material3_time_picker_uses_material_string_registry` installs a fake lookup and asserts localized
labels/values for:

- dial mode selector labels and spoken values,
- clock dial hour/minute labels,
- period group and AM/PM labels,
- input mode text-field labels and supporting text,
- modal scrim and action button labels.

`default_i18n_formats_material3_time_picker_strings` verifies bootstrap Fluent resources format both
plain and argument-backed Material3 TimePicker keys.

## Residual Risk

DatePicker locale-aware date descriptions remain open. That should be treated as a separate
DatePicker locale/date-formatting packet, not as unfinished TimePicker registry work.
