# Material 3 DatePicker Locale Strings Packet v1 - Handoff

Status: Closed
Last updated: 2026-05-28

DatePicker locale strings and date descriptions are closed for the current docked/modal calendar
surface.

The next Material3 work should return to the component matrix rather than continue inside this
packet. If future DatePicker modes are added, start a new packet for that mode instead of reopening
this one.

## Current Source Of Truth

- `ecosystem/fret-ui-material3/src/foundation/strings.rs`
- `ecosystem/fret-ui-material3/src/date_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `docs/workstreams/material3-date-picker-locale-strings-packet-v1/artifacts/date_picker_locale_strings_packet_v1.md`

## Known Follow-Ons

- Year selector, text input mode, and range picker remain future feature work if Material3 parity
  later requires those surfaces.
- Full locale/date formatting should stay app/i18n-backend-owned; Material helpers only pass
  structured arguments and English fallbacks.
