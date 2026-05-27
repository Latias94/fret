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

Localization remains intentionally open. The English fallback strings match Compose Material3's
English outcomes but are not yet routed through a Fret Material string registry.
