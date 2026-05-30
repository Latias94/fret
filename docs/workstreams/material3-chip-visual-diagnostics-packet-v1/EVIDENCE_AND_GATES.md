# Material 3 Chip Visual Diagnostics Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/chip.rs`
- `ecosystem/fret-ui-material3/src/suggestion_chip.rs`
- `ecosystem/fret-ui-material3/src/filter_chip.rs`
- `ecosystem/fret-ui-material3/src/input_chip.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/state_matrix.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-chip-visual-chrome.json`
- `tools/diag-scripts/ui-gallery-material3-chip-visual-chrome.json`
- `tools/diag-scripts/suites/ui-gallery-material3-chip-visual-chrome/suite.json`
- `docs/workstreams/material3-chip-visual-diagnostics-packet-v1/artifacts/chip_visual_diagnostics_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Canonical Gates

```powershell
python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-chip-visual-chrome.json
python -m json.tool tools/diag-scripts/ui-gallery-material3-chip-visual-chrome.json
python -m json.tool tools/diag-scripts/suites/ui-gallery-material3-chip-visual-chrome/suite.json
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-chip-visual-chrome.json --dir target/fret-diag/material3-chip-visual-chrome-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
target\debug\fretboard-dev.exe diag meta target\fret-diag\material3-chip-visual-chrome-20260528\sessions\1779935853792-64132\1779936147211 --json
target\debug\fretboard-dev.exe diag query test-id target\fret-diag\material3-chip-visual-chrome-20260528\sessions\1779935853792-64132\1779936147211 ui-gallery-material3-chip --json --top 80
target\debug\fretboard-dev.exe diag query test-id target\fret-diag\material3-chip-visual-chrome-20260528\sessions\1779935853792-64132\1779936147211 ui-gallery-material3-filter-chip --json --top 80
target\debug\fretboard-dev.exe diag query test-id target\fret-diag\material3-chip-visual-chrome-20260528\sessions\1779935853792-64132\1779936147211 ui-gallery-material3-input-chip --json --top 80
target\debug\fretboard-dev.exe diag query test-id target\fret-diag\material3-chip-visual-chrome-20260528\sessions\1779935853792-64132\1779936147211 ui-gallery-material3-suggestion-chip --json --top 80
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_segmented_buttons_and_chips_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment chips_export_checked_state_for_selected_semantics chip_set_roving_treats_trailing_action_focus_as_active_chip
python -m json.tool docs/workstreams/material3-chip-visual-diagnostics-packet-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json
python tools/check_diag_scripts_registry.py
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- Local verification set `TEMP`/`TMP` to `target/tmp` because the machine's `C:` temporary
  directory was full.
- Diagnostics run id: `1779936147211`.
- `diag meta` reported 3 snapshots, 303 unique test ids, and one window.
