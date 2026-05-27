# Material 3 DatePicker Selectable Dates Packet v1 - Handoff

Status: Closed
Last updated: 2026-05-28

## Current State

Selectable-date disabling is implemented and gated for `DockedDatePicker` and `DatePickerDialog`.
The public recipe API is `selectable_dates(|date| ...)` and defaults to all dates.

## Continue Policy

Return to the broader Material3 component alignment sweep. Remaining DatePicker follow-ons are
locale-specific labels and live-region announcements; those should be split only when the
accessibility truth set is explicit.
