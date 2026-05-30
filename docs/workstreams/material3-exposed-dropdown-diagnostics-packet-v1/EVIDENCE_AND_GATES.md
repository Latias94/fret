# Material 3 Exposed Dropdown Diagnostics Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/exposed_dropdown.rs`
- `ecosystem/fret-ui-material3/src/autocomplete.rs`
- `ecosystem/fret-ui-material3/src/text_field.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/autocomplete.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-exposed-dropdown-filtering.json`
- `tools/diag-scripts/suites/ui-gallery-material3-exposed-dropdown-filtering/suite.json`
- `docs/workstreams/material3-exposed-dropdown-diagnostics-packet-v1/artifacts/exposed_dropdown_diagnostics_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Canonical Gates

```powershell
python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-exposed-dropdown-filtering.json
python -m json.tool tools/diag-scripts/suites/ui-gallery-material3-exposed-dropdown-filtering/suite.json
python tools/check_diag_scripts_registry.py
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-exposed-dropdown-filtering.json --dir target/fret-diag/material3-exposed-dropdown-filtering-20260528-pass --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
target\debug\fretboard-dev.exe diag meta target\fret-diag\material3-exposed-dropdown-filtering-20260528-pass\sessions\1779944866704-59440\1779945219795 --json
target\debug\fretboard-dev.exe diag query test-id target\fret-diag\material3-exposed-dropdown-filtering-20260528-pass\sessions\1779944866704-59440\1779945241904-ui-gallery-material3-exposed-dropdown-filtering ui-gallery-material3-exposed-dropdown --json --top 160
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_exposed_dropdown_trailing_icon_toggles_overlay_v1
python -m json.tool docs/workstreams/material3-exposed-dropdown-diagnostics-packet-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- Local verification set `TEMP`/`TMP` to `target/tmp` because the machine's `C:` temporary
  directory can be full.
- Filtering diagnostics run id: `1779945219795`.
- `diag meta` reported 84 snapshots, 79 unique test ids, and a single UI Gallery window.
- The open filtering bundle
  `1779945241904-ui-gallery-material3-exposed-dropdown-filtering` found the root field, trailing
  icon, listbox, `option.gamma`, and `option.gamma.chrome` selectors. The final packed/closed
  bundle returns to the committed `beta` display after the popup is closed.
