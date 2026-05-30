# Material 3 TimePicker String Registry Packet v1 - Closeout Audit

Status: Closed
Date: 2026-05-28

## Decision

Closed as a Material foundation plus TimePicker recipe fix.

No `fret-ui` mechanism or `fret-ui-kit` policy work was required. The existing runtime i18n service,
message keys, and typed arguments already provide the hard contract surface.

## What Changed

- Added `foundation::strings` as the Material-owned lookup bridge over `I18nService`.
- Routed TimePicker title, mode toggle, selector labels, spoken hour/minute values, input labels,
  supporting/error text, period labels, scrim label, and action button labels through typed helpers.
- Added default bootstrap Fluent strings for `en-US` and `zh-CN`.
- Added an automation-surface test that injects a lookup and verifies registry strings across
  docked dial, docked input, and modal TimePicker surfaces.
- Updated the Material3 picker packet and component matrix to close the TimePicker localization
  follow-on.

## Evidence

- `ecosystem/fret-ui-material3/src/foundation/strings.rs`
- `ecosystem/fret-ui-material3/src/time_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `docs/workstreams/material3-time-picker-string-registry-packet-v1/artifacts/time_picker_string_registry_packet_v1.md`

## Residual Risk

DatePicker still needs locale-aware date descriptions. That is intentionally separate because it
needs date formatting policy, not just string-key routing.
