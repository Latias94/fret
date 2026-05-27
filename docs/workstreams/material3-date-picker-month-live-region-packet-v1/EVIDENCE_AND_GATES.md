# Material 3 DatePicker Month Live Region Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/date_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_picker_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
- `docs/workstreams/material3-date-picker-month-live-region-packet-v1/artifacts/date_picker_month_live_region_packet_v1.md`

## Canonical Gates

```powershell
cargo fmt --package fret-ui-material3 -- --check
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_month_label_is_polite_live_region
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_exposes_stable_part_test_ids
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-date-picker-month-live-region-packet-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- The focused automation test asserts the initial month label, live-region flags, next-month label
  update, and live-region persistence after navigation.
- The stable selector test covers both docked and modal month-label part ids.
