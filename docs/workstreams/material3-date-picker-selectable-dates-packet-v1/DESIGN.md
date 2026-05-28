# Material 3 DatePicker Selectable Dates Packet v1 - Design

Status: Closed
Last updated: 2026-05-28

## Problem

The picker packet left DatePicker with stable row/column and value-derived day-cell selectors, but
the grid still treated every visible date as selectable. Material 3 exposes selectable-date policy
as a first-class DatePicker state concern: blocked dates remain visible, expose disabled semantics,
and must not mutate the selected date.

This is a Material recipe gap, not a mechanism gap. Fret `PressableProps` already owns disabled
semantics, focusability, and pointer/key activation blocking.

## Target State

- `DockedDatePicker` and `DatePickerDialog` accept a date predicate that defaults to all dates.
- A blocked day remains visible and keeps its value-derived automation anchor.
- Blocked day cells are disabled/focus-skipped and cannot update the selected model.
- Dialog staging respects the same predicate, so a blocked day cannot be committed through OK.
- No shared kit policy or `crates/*` mechanism change is introduced.

## Truth Set

- Truth 1: A predicate-blocked docked day cell exposes disabled semantics.
- Truth 2: Activating a predicate-blocked docked day cell does not update selection.
- Truth 3: Activating an allowed docked day cell still updates selection.
- Truth 4: A predicate-blocked modal day cell does not update draft selection and therefore cannot
  commit through the dialog OK action.
- Truth 5: Value-derived date anchors remain live for disabled dates.

## Layer Mapping

- `ecosystem/fret-ui-material3/src/date_picker.rs`: Material recipe owns the selectable-date API,
  day-cell enabled state, disabled visual opacity, and model mutation guard via disabled pressables.
- `crates/fret-ui`: existing Pressable disabled semantics and hit-test/event blocking are reused.
- `ecosystem/fret-ui-kit`: no new policy is needed; the picker grid is not a reusable kit primitive
  yet.
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`: Material-facing proof covers docked
  and modal surfaces.

## Non-Goals

- Do not add a full `SelectableDates` struct with year predicates yet; a date predicate can already
  disable all dates in a year and keeps this API small.
- Do not solve locale-specific month/day labels in this slice.
- Do not add live-region announcements for selection changes in this slice.
- Do not port MUI X internals; the local Material UI mirror does not include the active X
  DatePicker source.

Status note (2026-05-28): locale-specific month/day labels were closed later by
`docs/workstreams/material3-date-picker-locale-strings-packet-v1/`.

## Upstream References

- Compose Material3 `DatePicker.kt`: `SelectableDates`, `isSelectableDate`,
  `isSelectableYear`, disabled `Day(enabled = enabled)`, and disabled-date tests.
- Material Design 3 intent: disabled dates remain visible but unavailable for selection.
