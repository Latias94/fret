# Material 3 TimePicker Input Error Packet v1 - Design

Status: Closed
Last updated: 2026-05-28

## Problem

The picker packet left TimePicker input mode with stable field selectors and keyboard entry, but
invalid typed values were clamped into the committed time. That meant `27` in a 24h hour field
became `23` instead of staying as invalid input with error supporting text.

Compose Material3 keeps editable input state separate from the committed `TimePickerState` value:
invalid hour/minute input can remain visible, expose error chrome and supporting text, and must not
pollute the committed time.

This is a Material recipe gap. Fret already has `SemanticsInvalid`, live-region semantics, and
pressable text-field roles, so no `crates/*` mechanism change is needed.

## Target State

- TimePicker input mode validates the editable hour/minute strings before committing time changes.
- Invalid hour/minute strings remain visible and do not clamp into committed `Time`.
- Invalid fields expose `aria-invalid`-equivalent semantics.
- Supporting text switches between normal labels and Material error text:
  - `Hour`
  - `Minute`
  - `Hour must be 0-23`
  - `Hour must be 1-12`
  - `Minute must be 0-59`
- Supporting text has stable part selectors and polite atomic live-region semantics.
- Error colors are token-resolved at the Material time-input recipe layer with system color
  fallback.

## Truth Set

- Truth 1: Typing a valid first hour digit still updates committed time.
- Truth 2: Completing an invalid 24h hour value does not clamp or overwrite committed time.
- Truth 3: The invalid field exposes `SemanticsInvalid::True`.
- Truth 4: The hour supporting text exposes the 24h error label and a polite atomic live region.
- Truth 5: Deleting the invalid value and typing a valid value clears invalid semantics and updates
  committed time.
- Truth 6: Existing two-digit input replacement and auto-advance behavior remains intact.

## Layer Mapping

- `ecosystem/fret-ui-material3/src/time_picker.rs`: owns editable time input validity, supporting
  error text, field invalid semantics, and committed-time update rules.
- `ecosystem/fret-ui-material3/src/tokens/time_input.rs`: owns Material time-input error color
  resolution with system token fallback.
- `crates/fret-ui` / `crates/fret-core`: reused existing text, live-region, and invalid semantics.
- `ecosystem/fret-ui-kit`: no new shared policy is needed.
- Diagnostics/automation owns stable selector coverage for input supporting text parts.

## Non-Goals

- Do not add a full localized string registry in this slice.
- Do not change dial behavior.
- Do not introduce a shared text-field primitive under `fret-ui-kit`; this is still TimePicker
  recipe-specific state.
- Do not port unavailable MUI X internals; Compose Material3 is the semantic source of truth here.

## Upstream References

- Compose Material3 `TimePicker.kt`: `hourInput`, `minuteInput`, `isHourInputValid`,
  `isMinuteInputValid`, `SupportingText`, `LiveRegionMode.Polite`, and invalid input tests.
- Compose English strings: `TimePickerMinuteError`, `TimePickerHourError`,
  `TimePicker24HourError`.
