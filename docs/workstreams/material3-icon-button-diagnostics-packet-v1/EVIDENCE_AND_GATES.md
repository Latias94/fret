# Material 3 IconButton Diagnostics Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/icon_button.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/icon_button.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-icon-button-centered-chrome.json`
- `docs/workstreams/material3-icon-button-diagnostics-packet-v1/artifacts/icon_button_diagnostics_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Canonical Gates

```powershell
python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-icon-button-centered-chrome.json
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-icon-button-centered-chrome.json --dir target/fret-diag/material3-icon-button-centered-chrome-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
target\debug\fretboard-dev.exe diag meta target\fret-diag\material3-icon-button-centered-chrome-20260528\sessions\1779937486360-34444\1779937783108 --json
target\debug\fretboard-dev.exe diag query test-id target\fret-diag\material3-icon-button-centered-chrome-20260528\sessions\1779937486360-34444\1779937783108 ui-gallery-material3-icon --json --top 80
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_choice_controls_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment icon_button_pressed_scene_structure_is_stable
python -m json.tool docs/workstreams/material3-icon-button-diagnostics-packet-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json
python tools/check_diag_scripts_registry.py
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- Local verification set `TEMP`/`TMP` to `target/tmp` because the machine's `C:` temporary
  directory was full.
- Diagnostics run id: `1779937783108`.
- `diag meta` reported 28 snapshots, 54 unique test ids, and one window.
