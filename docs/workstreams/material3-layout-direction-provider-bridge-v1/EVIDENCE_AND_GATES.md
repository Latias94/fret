# Material3 Layout Direction Provider Bridge v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-30

## Repro

- Material3 context direction provider:
  - `cargo nextest run -p fret-ui-material3 --lib layout_direction_inherits_masks_and_restores material_resolved_layout_direction_provides_core_direction_for_elements`
- Tabs RTL physical order:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state rtl_tabs_theme_direction_mirrors_physical_tab_order`

## Gates

- `cargo fmt -p fret-ui-material3`
- `cargo nextest run -p fret-ui-material3 --lib layout_direction_inherits_masks_and_restores material_resolved_layout_direction_provides_core_direction_for_elements`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state rtl_tabs_theme_direction_mirrors_physical_tab_order`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- `python -m json.tool docs/workstreams/material3-layout-direction-provider-bridge-v1/WORKSTREAM.json | Out-Null`
- `python tools/check_workstream_catalog.py`
- `python tools/check_layering.py`
- `git diff --check`

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/foundation/context.rs`
- `ecosystem/fret-ui-material3/src/tabs.rs`
- `ecosystem/fret-ui-material3/tests/tabs_state.rs`
- `crates/fret-ui/src/elements/cx.rs`
- `crates/fret-ui/src/tree/layout/flex.rs`

## Verified On 2026-05-30

- `cargo fmt -p fret-ui-material3`
- `cargo nextest run -p fret-ui-material3 --lib layout_direction_inherits_masks_and_restores material_resolved_layout_direction_provides_core_direction_for_elements`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state rtl_tabs_theme_direction_mirrors_physical_tab_order`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- `python -m json.tool docs\workstreams\material3-layout-direction-provider-bridge-v1\WORKSTREAM.json | Out-Null`
- `python tools\check_workstream_catalog.py`
- `python tools\check_layering.py`
- `git diff --check`
