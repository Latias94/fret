# Material3 Tabs RTL Indicator v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-30

## Evidence Anchors

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TabRow.kt`
- `ecosystem/fret-ui-material3/src/foundation/context.rs`
- `ecosystem/fret-ui-material3/src/tabs.rs`
- `ecosystem/fret-ui-material3/src/lib.rs`
- `ecosystem/fret-ui-material3/tests/tabs_state.rs`

## Gates

- `cargo fmt -p fret-ui-material3`
- `cargo nextest run -p fret-ui-material3 --lib tab_keyboard_direction_maps_arrow_keys_by_layout_direction tab_indicator_fallback_position_mirrors_logical_index_in_rtl`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state rtl_tabs_arrow_left_moves_to_next_logical_tab_without_wrapping`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- `python -m json.tool docs/workstreams/material3-tabs-rtl-indicator-v1/WORKSTREAM.json | Out-Null`
- `python tools/check_workstream_catalog.py`

## Residuals

- Fret Flex layout does not yet expose a global RTL physical placement contract.
- Scrollable TabRow selected-tab auto-scroll remains a separate Material3 parity slice.
