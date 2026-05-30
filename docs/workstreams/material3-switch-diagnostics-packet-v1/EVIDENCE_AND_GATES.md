# Material 3 Switch Diagnostics Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/switch.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/switch.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-switch-icons-state-matrix-screenshots.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_switch_adapter_report_v1.json`
- `docs/workstreams/material3-switch-diagnostics-packet-v1/artifacts/switch_diagnostics_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Canonical Gates

```powershell
python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_switch_adapter_report_v1.json
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-switch-icons-state-matrix-screenshots.json --dir target/fret-diag/material3-switch-icons-state-matrix-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
target\debug\fretboard-dev.exe diag meta target\fret-diag\material3-switch-icons-state-matrix-20260528\sessions\1779936902105-61844\1779937207775 --json
target\debug\fretboard-dev.exe diag query test-id target\fret-diag\material3-switch-icons-state-matrix-20260528\sessions\1779936902105-61844\1779937207775 ui-gallery-material3-switch --json --top 100
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_switch_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment switch_ripple_origin_tracks_pointer_down_position switch_ripple_holds_for_minimum_press_duration_before_fade
python -m json.tool docs/workstreams/material3-switch-diagnostics-packet-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- Local verification set `TEMP`/`TMP` to `target/tmp` because the machine's `C:` temporary
  directory was full.
- Diagnostics run id: `1779937207775`.
- `diag meta` reported 193 snapshots, 85 unique test ids, and one window.
- The Switch adapter report summary has `part_count = 5`, `pass_known = 5`, and no top findings.
