# Evidence and Gates - Material 3 NavigationDrawer Selector Completion Packet v1

Status: Closed

## Canonical Evidence

- `ecosystem/fret-ui-material3/src/navigation_drawer.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
- `docs/workstreams/material3-navigation-drawer-selector-completion-packet-v1/artifacts/navigation_drawer_selector_completion_packet_v1.md`

## Planned Gates

```powershell
cargo fmt --package fret-ui-material3 -- --check
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_navigation_drawer_exposes_stable_part_test_ids material3_modal_navigation_drawer_exposes_stable_part_test_ids
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-navigation-drawer-selector-completion-packet-v1/WORKSTREAM.json > $null
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null
python tools/check_workstream_catalog.py
git diff --check
```

## Verified Gates

Last verified: 2026-05-28

```powershell
cargo fmt --package fret-ui-material3 -- --check
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_navigation_drawer_exposes_stable_part_test_ids material3_modal_navigation_drawer_exposes_stable_part_test_ids
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-navigation-drawer-selector-completion-packet-v1/WORKSTREAM.json > $null
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null
python tools/check_workstream_catalog.py
git diff --check
```
