# Material 3 DatePicker Locale Strings Packet v1 - Closeout Audit

Status: Closed
Date: 2026-05-28

## Outcome

Closed the DatePicker locale/string/date-description follow-on without changing `fret-ui` mechanism
contracts.

## What Changed

- Added Material DatePicker string helpers over `I18nService` with English fallbacks.
- Routed DatePicker title, scrim, actions, month navigation, month/year labels, weekday labels, and
  day-cell descriptions through those helpers.
- Added `Button::a11y_label` so compact visible labels can expose fuller accessibility labels.
- Added bootstrap `en-US` and `zh-CN` Fluent defaults for DatePicker strings.
- Added focused DatePicker registry coverage for docked and modal surfaces.

## Owner Classification

- `material_foundation`: string key bridge, fallbacks, month/weekday helpers, date-description
  argument shaping.
- `material_recipe`: DatePicker consumption, day-cell role/label/selected semantics, modal title
  and scrim semantics, compact nav labels.
- `fret-bootstrap`: default Fluent resources.
- `fret-ui` / `fret-ui-kit`: no new mechanism or policy surface needed.

## Evidence

- `ecosystem/fret-ui-material3/src/foundation/strings.rs`
- `ecosystem/fret-ui-material3/src/button.rs`
- `ecosystem/fret-ui-material3/src/date_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `docs/workstreams/material3-date-picker-locale-strings-packet-v1/artifacts/date_picker_locale_strings_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_picker_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Gates

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_uses_material_string_registry_and_date_descriptions
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker
cargo test -p fret-bootstrap --lib default_i18n_formats_material3_date_picker_strings
```

Broader check/clippy/catalog gates are tracked in `EVIDENCE_AND_GATES.md`.

## Residual Risk

No DatePicker locale-string residual remains for the current docked/modal calendar grid. Unbuilt
Material picker modes should be handled as separate feature packets.
