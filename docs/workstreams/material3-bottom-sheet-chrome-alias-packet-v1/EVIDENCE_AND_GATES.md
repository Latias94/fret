# Material 3 BottomSheet Chrome Alias Packet v1 - Evidence And Gates

Status: Active
Last updated: 2026-05-28

## Current Evidence

- `material3_overlay_feedback_packet_v1.md` explicitly withheld bottom-sheet `.chrome` aliases
  because layout-sensitive markers changed sizing.
- `foundation::test_id::diagnostic_anchor` now provides hidden, non-focusable layout-only markers.
- `bottom_sheet.rs` currently exposes root/sheet/drag-handle and scrim ids but not root/sheet
  chrome aliases.

## Gate Set

```powershell
python -m json.tool docs/workstreams/material3-bottom-sheet-chrome-alias-packet-v1/WORKSTREAM.json | Out-Null
python tools/check_workstream_catalog.py
cargo fmt --package fret-ui-material3 -- --check
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_dialog_and_bottom_sheet_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_bottom_sheet_suite_goldens_v1
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

## Evidence Log

- 2026-05-28: Opened the lane from the closed overlay/feedback packet follow-on.
- 2026-05-28: M3BS-020 red/green selector proof.
  - Red: `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_dialog_and_bottom_sheet_expose_stable_part_test_ids`
    failed on missing `m3-bottom-sheet.chrome`.
  - Green: the same focused automation-surface test passed after adding hidden full-region anchors
    to `DockedBottomSheet`.
- 2026-05-28: M3BS-030 closeout verification.
  - `cargo fmt --package fret-ui-material3 -- --check`: passed.
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_dialog_and_bottom_sheet_expose_stable_part_test_ids`:
    passed.
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_bottom_sheet_suite_goldens_v1`:
    passed without golden refresh.
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`:
    passed.
  - `python -m json.tool docs/workstreams/material3-bottom-sheet-chrome-alias-packet-v1/WORKSTREAM.json | Out-Null`:
    passed.
  - `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json | Out-Null`:
    passed.
  - `python tools/check_workstream_catalog.py`: passed, 479 dedicated directories and 47
    standalone markdown files.
