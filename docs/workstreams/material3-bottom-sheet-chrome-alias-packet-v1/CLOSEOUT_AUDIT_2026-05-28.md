# Material 3 BottomSheet Chrome Alias Packet v1 - Closeout Audit

Date: 2026-05-28
Status: Closed

## Result

The lane is closed.

Bottom sheets now expose layout-safe chrome aliases:

- `bottom_sheet.chrome`
- `modal_bottom_sheet.sheet.chrome`

The implementation uses hidden, non-focusable diagnostic anchors and does not change scene output.

## Gate Evidence

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_dialog_and_bottom_sheet_expose_stable_part_test_ids`:
  passed.
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_bottom_sheet_suite_goldens_v1`:
  passed without golden refresh.
- `cargo fmt --package fret-ui-material3 -- --check`: passed.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`:
  passed.
- `python -m json.tool docs/workstreams/material3-bottom-sheet-chrome-alias-packet-v1/WORKSTREAM.json | Out-Null`:
  passed.
- `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json | Out-Null`:
  passed.
- `python tools/check_workstream_catalog.py`: passed, 479 dedicated directories and 47 standalone
  markdown files.

## Boundary Notes

- No overlay policy changed.
- No `crates/*` contract changed.
- No bottom-sheet golden refresh was needed.
