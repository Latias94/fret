# Material 3 DatePicker Month Live Region Packet v1

Status: closed
Date: 2026-05-28

## Truth

- The displayed month/year label is a polite atomic live region.
- Month navigation updates the same label.
- Docked and modal month labels have stable part ids.

## Artifacts

- `ecosystem/fret-ui-material3/src/date_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-date-picker-month-live-region-packet-v1/`

## Wiring

- `month_nav_header` derives `docked.month-label` or `modal.month-label` from the picker base id.
- The text element keeps its label text and attaches `SemanticsLive::Polite` with
  `live_atomic = true`.
- Existing previous/next month actions update the month model and therefore update the live-region
  label on the next render.

## Proof

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_month_label_is_polite_live_region
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_exposes_stable_part_test_ids
```

## Residual Risk

Locale-aware month names and full date descriptions were closed by
`docs/workstreams/material3-date-picker-locale-strings-packet-v1/`.
