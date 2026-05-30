# DatePicker Locale Strings Packet v1

## Truth

- DatePicker visible labels and automation-facing semantics no longer depend on recipe-local English
  strings.
- `foundation::strings` owns the Material DatePicker string-key bridge and falls back to English
  Material outcomes when no `I18nService` backend/key is present.
- Docked and modal DatePicker surfaces consume the same Material string helpers.
- Bootstrap default i18n resources include DatePicker strings for `en-US` and `zh-CN`, including
  argument-backed month/year and day-description strings.

## Artifacts

- `ecosystem/fret-ui-material3/src/foundation/strings.rs`
- `ecosystem/fret-ui-material3/src/button.rs`
- `ecosystem/fret-ui-material3/src/date_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Wiring

The registry uses `fret_runtime::fret_i18n::I18nService` from the host global store. Missing service
or missing keys return English fallback strings, so existing apps keep their previous behavior while
localized apps can override Material strings through the normal Fret i18n path.

Date descriptions pass structured `year`, `month`, `month_number`, and `day` arguments. The
Material fallback provides English output; app-level lookup resources can reorder or format these
arguments for another locale.

## Proof

`material3_date_picker_uses_material_string_registry_and_date_descriptions` installs a fake lookup
and asserts localized labels/descriptions for:

- docked month label and month navigation,
- docked weekday long labels,
- today and selected day-cell descriptions,
- modal scrim, title, month label, weekday labels, navigation labels, and actions.

`default_i18n_formats_material3_date_picker_strings` verifies bootstrap Fluent resources format both
plain and argument-backed Material3 DatePicker keys.

## Residual Risk

No DatePicker locale-string residual remains for the current docked/modal calendar grid. Future
Material picker modes, if added, need their own parity packet.
