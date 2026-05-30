# Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/search_bar.rs`
- `ecosystem/fret-ui-material3/src/tokens/search_bar.rs`
- `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs`
- `ecosystem/fret-ui-material3/src/tokens/v30.rs`
- `ecosystem/fret-ui-material3/src/bin/material3_token_import.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_field_family_selector_audit_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_field_family_behavior_packet_v1.md`
- `docs/workstreams/material3-search-view-state-packet-v1/artifacts/search_view_source_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
- `goldens/material3-headless/v1/material3-search-bar.*.json`

## Canonical Gates

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_bar_suite_goldens_v1
python -m json.tool docs/workstreams/material3-search-bar-headless-packet-v1/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json | Out-Null
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- Both focused nextest gates passed on 2026-05-28.
- The lane intentionally has no standalone gallery diagnostic script.
- The machine may need `TEMP`/`TMP` redirected to `target/tmp` when the `C:` temp directory is
  full.
