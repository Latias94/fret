# Material 3 TimePicker Input Error Packet v1

Status: closed
Date: 2026-05-28

## Truth

- Editable TimePicker input state may be invalid without changing committed time.
- Invalid fields expose error semantics and supporting text.
- Supporting text updates are polite live-region announcements.

## Artifacts

- `ecosystem/fret-ui-material3/src/time_picker.rs`
- `ecosystem/fret-ui-material3/src/tokens/time_input.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-time-picker-input-error-packet-v1/`

## Wiring

- `time_input_field_is_valid` classifies editable hour/minute strings without mutating committed
  time.
- `apply_time_input_models` commits only valid parsed fields and leaves invalid fields staged.
- `time_input_field` maps invalid state to `SemanticsInvalid::True` and error token colors.
- `time_input_field_column` exposes `input.<field>.supporting-text` ids and live-region semantics.

## Proof

```powershell
cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_time_input_rejects_invalid_values_and_recovers
cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_time_input_replaces_and_auto_advances_hour
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids
```

## Residual Risk

Localized TimePicker strings and broader selection/mode-change announcements remain open
accessibility-depth follow-ons.
