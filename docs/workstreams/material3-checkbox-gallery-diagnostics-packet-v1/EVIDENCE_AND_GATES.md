# Material 3 Checkbox Gallery Diagnostics Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/checkbox.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/checkbox.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-checkbox-centered-chrome.json`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-checkbox-tristate-screenshots.json`
- `docs/workstreams/material3-checkbox-gallery-diagnostics-packet-v1/artifacts/checkbox_gallery_diagnostics_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Canonical Gates

```powershell
python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-checkbox-centered-chrome.json
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-checkbox-centered-chrome.json --dir target/fret-diag/material3-checkbox-centered-chrome-20260528-fixed --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-checkbox-tristate-screenshots.json --dir target/fret-diag/material3-checkbox-tristate-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
target\debug\fretboard-dev.exe diag meta target\fret-diag\material3-checkbox-centered-chrome-20260528-fixed\sessions\1779934722696-58708\1779935007417 --json
target\debug\fretboard-dev.exe diag query test-id target\fret-diag\material3-checkbox-centered-chrome-20260528-fixed\sessions\1779934722696-58708\1779935007417 ui-gallery-material3-checkbox --json --top 60
target\debug\fretboard-dev.exe diag meta target\fret-diag\material3-checkbox-tristate-20260528\sessions\1779935052939-58744\1779935349931 --json
target\debug\fretboard-dev.exe diag query test-id target\fret-diag\material3-checkbox-tristate-20260528\sessions\1779935052939-58744\1779935349931 ui-gallery-material3-checkbox --json --top 80
cargo nextest run -p fret-ui-material3 --test radio_alignment checkbox_pressed_scene_structure_is_stable checkbox_tristate_semantics_and_toggle_outcomes
python -m json.tool docs/workstreams/material3-checkbox-gallery-diagnostics-packet-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- Local verification set `TEMP`/`TMP` to `target/tmp` because the machine's `C:` temporary
  directory was full.
- Pre-fix centered-chrome run `1779934611432` failed waiting for `ui-gallery-material3-checkbox`
  on the aggregate Material3 gallery page.
- Fixed centered-chrome run `1779935007417` passed.
- Tri-state run `1779935349931` passed.
- Both fixed bundles reported one window and 60 unique test ids.
