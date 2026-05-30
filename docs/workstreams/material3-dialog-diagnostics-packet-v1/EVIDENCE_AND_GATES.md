# Material 3 Dialog Diagnostics Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/dialog.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/dialog.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-dialog-focus-trap-restore.json`
- `tools/diag-scripts/suites/ui-gallery-material3-dialog-focus-trap-restore/suite.json`
- `docs/workstreams/material3-dialog-diagnostics-packet-v1/artifacts/dialog_diagnostics_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Canonical Gates

```powershell
python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-dialog-focus-trap-restore.json
python tools/check_diag_scripts_registry.py
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-dialog-focus-trap-restore.json --dir target/fret-diag/material3-dialog-focus-trap-restore-20260528-final --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
target\debug\fretboard-dev.exe diag meta target\fret-diag\material3-dialog-focus-trap-restore-20260528-final\sessions\1779939582503-8856\1779939874070 --json
target\debug\fretboard-dev.exe diag query test-id target\fret-diag\material3-dialog-focus-trap-restore-20260528-final\sessions\1779939582503-8856\1779939874070 ui-gallery-material3-dialog --json --top 160
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_dialog_and_bottom_sheet_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment dialog_focus_is_contained_and_restored_across_schemes
cargo nextest run -p fret-ui-material3 --test radio_alignment dialog_scrim_dismisses_without_activating_underlay
cargo nextest run -p fret-ui-material3 --test radio_alignment dialog_style_overrides_apply_to_container_and_text
python -m json.tool docs/workstreams/material3-dialog-diagnostics-packet-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- Local verification set `TEMP`/`TMP` to `target/tmp` because the machine's `C:` temporary
  directory can be full.
- Diagnostics run id: `1779939874070`.
- `diag meta` reported 10 snapshots, 75 unique test ids, and one window.
- The bundle query found Dialog open/action/select/scrim/panel selectors, including
  `ui-gallery-material3-dialog.panel`, `ui-gallery-material3-dialog.panel.chrome`,
  `ui-gallery-material3-dialog.scrim`, and `ui-gallery-material3-dialog.scrim.chrome`.
