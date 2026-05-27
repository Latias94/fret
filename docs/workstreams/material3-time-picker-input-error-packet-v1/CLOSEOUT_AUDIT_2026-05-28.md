# Material 3 TimePicker Input Error Packet v1 - Closeout Audit

Status: Closed
Date: 2026-05-28

## Decision

Closed as a Material recipe fix.

No `fret-ui` or `fret-ui-kit` mechanism work was required because the needed primitives already
exist: text semantics, live-region flags, pressable text-field roles, and `SemanticsInvalid`.

## What Changed

- `apply_time_input_models` validates typed values before writing committed time.
- Invalid values stay in the editable model and are rendered as error state.
- Time input fields set invalid semantics only while the current editable value is invalid.
- Supporting text exposes Material error labels, stable part ids, and polite atomic live-region
  semantics.
- Token helpers now resolve time-input error container/label/outline/supporting colors with system
  fallback.

## Evidence

- `ecosystem/fret-ui-material3/src/time_picker.rs`
- `ecosystem/fret-ui-material3/src/tokens/time_input.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-time-picker-input-error-packet-v1/artifacts/time_picker_input_error_packet_v1.md`

## Residual Risk

Localization was intentionally open in this packet and is now closed by
`docs/workstreams/material3-time-picker-string-registry-packet-v1/`. The English fallback strings
still match Compose Material3 outcomes when no app i18n backend is installed.
