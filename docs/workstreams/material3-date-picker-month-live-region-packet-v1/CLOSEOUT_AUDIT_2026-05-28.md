# Material 3 DatePicker Month Live Region Packet v1 - Closeout Audit

Status: Closed
Date: 2026-05-28

## Decision

Closed as a Material recipe semantics fix.

Compose Material3 marks the displayed month/year text as a polite live region. Fret can express the
same outcome with existing `SemanticsDecoration` and `SemanticsLive`; no mechanism change is
required.

## What Changed

- Month header text now exposes stable docked/modal `month-label` part ids.
- Month header text is a polite atomic live region.
- Automation verifies initial label text, label update after next-month activation, and live-region
  persistence.

## Evidence

- `ecosystem/fret-ui-material3/src/date_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-date-picker-month-live-region-packet-v1/artifacts/date_picker_month_live_region_packet_v1.md`

## Residual Risk

The month names remain English-only through the existing `month_name_en` helper. Localization should
be a separate DatePicker formatter/string-registry slice.
