# Material 3 DatePicker Selectable Dates Packet v1

Status: closed
Date: 2026-05-28

## Truth

- Material DatePicker lets callers block dates from selection.
- Blocked dates remain visible but expose disabled semantics and do not activate.
- The same day grid behavior applies to docked and dialog pickers.

## Artifacts

- `ecosystem/fret-ui-material3/src/date_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-date-picker-selectable-dates-packet-v1/`

## Wiring

- `DockedDatePicker::selectable_dates(|date| ...)` stores a cloneable recipe-level predicate.
- `DatePickerDialog::selectable_dates(|date| ...)` forwards the same predicate into the modal
  panel and staged day grid.
- `dates_grid` maps predicate false to disabled/focus-skipped pressables and disabled label
  opacity while preserving the value-derived date anchor.

## Proof

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_respects_selectable_dates`

## Residual Risk

Locale-specific labels and live-region announcements were closed later by dedicated DatePicker
packets.
