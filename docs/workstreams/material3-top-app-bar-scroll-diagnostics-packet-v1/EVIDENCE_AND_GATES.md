# Material 3 TopAppBar Scroll Diagnostics Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/top_app_bar.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-top-app-bar-scroll-screenshots.json`
- `docs/workstreams/material3-top-app-bar-scroll-diagnostics-packet-v1/artifacts/top_app_bar_scroll_diagnostics_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_surface_data_display_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Canonical Gates

```powershell
cargo run -p fretboard-dev -- diag config doctor --mode launch --print-launch-policy
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-top-app-bar-scroll-screenshots.json --dir target/fret-diag/material3-top-app-bar-scroll-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
target\debug\fretboard-dev.exe diag meta target\fret-diag\material3-top-app-bar-scroll-20260528\sessions\1779933189257-62812\1779933454871 --json
target\debug\fretboard-dev.exe diag query test-id target\fret-diag\material3-top-app-bar-scroll-20260528\sessions\1779933189257-62812\1779933454871 top-app-bar --json --top 80
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_top_app_bar_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --test top_app_bar_alignment top_app_bar_exposes_toolbar_semantics_role
python -m json.tool docs/workstreams/material3-top-app-bar-scroll-diagnostics-packet-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- Local verification set `TEMP`/`TMP` to `target/tmp` because the machine's `C:` temporary directory
  was full.
- Diagnostics run id: `1779933454871`.
- AI packet:
  `target/fret-diag/material3-top-app-bar-scroll-20260528/sessions/1779933189257-62812/1779933454871/ai.packet`.
- Share zip:
  `target/fret-diag/material3-top-app-bar-scroll-20260528/sessions/1779933189257-62812/share/1779933454871.zip`.
- `diag meta` reported `snapshots_total = 300`, `total_unique_test_ids = 135`, and one window.
