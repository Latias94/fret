# Material 3 Menu And Dropdown Diagnostics Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/menu.rs`
- `ecosystem/fret-ui-material3/src/dropdown_menu.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/menu.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-menu-focus-dismiss.json`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-menu-item-chrome-fill.json`
- `docs/workstreams/material3-menu-dropdown-diagnostics-packet-v1/artifacts/menu_dropdown_diagnostics_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Canonical Gates

```powershell
python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-menu-focus-dismiss.json
python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-menu-item-chrome-fill.json
python tools/check_diag_scripts_registry.py
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-menu-focus-dismiss.json --dir target/fret-diag/material3-menu-focus-dismiss-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-menu-item-chrome-fill.json --dir target/fret-diag/material3-menu-item-chrome-fill-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
target\debug\fretboard-dev.exe diag meta target\fret-diag\material3-menu-focus-dismiss-20260528\sessions\1779940661967-71588\1779940975756 --json
target\debug\fretboard-dev.exe diag query test-id target\fret-diag\material3-menu-focus-dismiss-20260528\sessions\1779940661967-71588\1779940975756 ui-gallery-material3-menu --json --top 160
target\debug\fretboard-dev.exe diag meta target\fret-diag\material3-menu-item-chrome-fill-20260528\sessions\1779941051623-57416\1779941390986 --json
target\debug\fretboard-dev.exe diag query test-id target\fret-diag\material3-menu-item-chrome-fill-20260528\sessions\1779941051623-57416\1779941390986 ui-gallery-material3-menu-item --json --top 160
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_menu_and_dropdown_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment dropdown_menu_dismisses_and_restores_focus_across_schemes
cargo nextest run -p fret-ui-material3 --test radio_alignment menu_pressed_scene_structure_is_stable
cargo nextest run -p fret-ui-material3 --test radio_alignment menu_style_overrides_apply_to_container_and_label
python -m json.tool docs/workstreams/material3-menu-dropdown-diagnostics-packet-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json
python tools/check_workstream_catalog.py
git diff --check
```

## Verification Notes

- Local verification set `TEMP`/`TMP` to `target/tmp` because the machine's `C:` temporary
  directory can be full.
- Focus/dismiss diagnostics run id: `1779940975756`; `diag meta` reported 10 snapshots, 63 unique
  test ids, and one window.
- Chrome-fill diagnostics run id: `1779941390986`; `diag meta` reported 47 snapshots, 63 unique
  test ids, and one window.
- Bundle queries found default and override menu roots plus item `.chrome` selectors.
