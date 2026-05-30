# Material 3 Segmented Button Diagnostics Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/segmented_button.rs`
- `ecosystem/fret-ui-material3/src/tokens/segmented_button.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/segmented_button.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-segmented-button-roving-semantics-screenshots.json`
- `tools/diag-scripts/suites/ui-gallery-material3-segmented-button-roving-semantics-screenshots/suite.json`
- `docs/workstreams/material3-segmented-button-diagnostics-packet-v1/artifacts/segmented_button_diagnostics_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Canonical Gates

```powershell
python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-segmented-button-roving-semantics-screenshots.json
python -m json.tool tools/diag-scripts/suites/ui-gallery-material3-segmented-button-roving-semantics-screenshots/suite.json
python tools/check_diag_scripts_registry.py
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-segmented-button-roving-semantics-screenshots.json --dir target/fret-diag/material3-segmented-button-roving-semantics-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
target\debug\fretboard-dev.exe diag meta target\fret-diag\material3-segmented-button-roving-semantics-20260528\sessions\1779945893709-62064\1779946252252 --json
target\debug\fretboard-dev.exe diag query test-id target\fret-diag\material3-segmented-button-roving-semantics-20260528\sessions\1779945893709-62064\1779946252252 ui-gallery-material3-segmented-single --json --top 120
cargo nextest run -p fret-ui-material3 --test radio_alignment segmented_button_semantics_roles_match_compose_baseline
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_segmented_button_suite_goldens_v1
python -m json.tool docs/workstreams/material3-segmented-button-diagnostics-packet-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- Local verification set `TEMP`/`TMP` to `target/tmp` because the machine's `C:` temporary
  directory can be full.
- Diagnostics run id: `1779946252252`.
- `diag meta` reported 57 snapshots, 63 unique test ids, and a single UI Gallery window.
- The roving-semantics bundle exposed stable single-select item ids and checked state across the
  single-select group.
