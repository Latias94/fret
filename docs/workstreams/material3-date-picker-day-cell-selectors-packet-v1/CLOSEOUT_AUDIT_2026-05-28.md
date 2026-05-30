# Closeout Audit - Material 3 DatePicker Day Cell Selectors Packet v1

Date: 2026-05-28
Status: Closed

## Scope Audit

Closed:

- DatePicker rendered day cells expose hidden `cell.<yyyy-mm-dd>` diagnostic anchors.
- Existing `cell.<row>.<col>` semantic ids remain available and unchanged.
- Docked and modal DatePicker render paths are covered by focused automation assertions.
- The component matrix and picker packet record the selector improvement and keep larger
  accessibility work split out.

Not closed by this packet:

- `SelectableDates` or disabled-date policy.
- Localized spoken date labels.
- Live-region month/year announcements.

## Evidence

- `ecosystem/fret-ui-material3/src/date_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_picker_packet_v1.md`
- `docs/workstreams/material3-date-picker-day-cell-selectors-packet-v1/artifacts/date_picker_day_cell_selectors_packet_v1.md`

## Verified Gates

```powershell
cargo fmt --package fret-ui-material3 -- --check
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_exposes_stable_part_test_ids
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-date-picker-day-cell-selectors-packet-v1/WORKSTREAM.json > $null
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null
python tools/check_workstream_catalog.py
git diff --check
```

## Follow-On Recommendation

The next DatePicker accessibility packet should handle `SelectableDates` and disabled-date semantics
before live-region month announcements, because enabled/disabled day state is the harder behavioral
contract.

