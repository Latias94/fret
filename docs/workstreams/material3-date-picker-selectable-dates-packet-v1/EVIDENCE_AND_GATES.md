# Material 3 DatePicker Selectable Dates Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/date_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_picker_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
- `docs/workstreams/material3-date-picker-selectable-dates-packet-v1/artifacts/date_picker_selectable_dates_packet_v1.md`

## Canonical Gates

```powershell
cargo fmt --package fret-ui-material3 -- --check
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_respects_selectable_dates
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_exposes_stable_part_test_ids
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-date-picker-selectable-dates-packet-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- The focused automation test clicks a blocked docked day and verifies the selected model remains
  `None`, then clicks an allowed docked day and verifies selection updates.
- The same test opens the modal dialog, clicks a blocked day, confirms, and verifies no selected
  date is committed.
- Disabled semantics are asserted through the row/column day-cell semantics ids, while the
  value-derived date anchor is still asserted live for the blocked date.
