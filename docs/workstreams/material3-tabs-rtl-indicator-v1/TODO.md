# Material3 Tabs RTL Indicator v1 - TODO

Status: Closed
Last updated: 2026-05-30

Task IDs use `M3TABRTL-*`.

## M0 - Lane Setup

- [x] M3TABRTL-010 [owner=codex] [deps=none] [scope=docs/workstreams/material3-tabs-rtl-indicator-v1]
  Goal: Split Tabs RTL indicator/navigation into a narrow follow-on.
  Validation: `python -m json.tool docs/workstreams/material3-tabs-rtl-indicator-v1/WORKSTREAM.json | Out-Null`; `python tools/check_workstream_catalog.py`.
  Review: DONE. The lane records Compose `LayoutDirection` indicator handling and the existing
  Material3 component direction pattern as the source stack.
  Handoff: Start M3TABRTL-020.

## M1 - Direction-Aware Tabs

- [x] M3TABRTL-020 [owner=codex] [deps=M3TABRTL-010] [scope=ecosystem/fret-ui-material3/src/{tabs.rs,lib.rs},ecosystem/fret-ui-material3/tests/tabs_state.rs]
  Goal: Resolve Material layout direction in Tabs, flip horizontal roving navigation in RTL, mirror
  indicator fallback positioning, and expose public Material layout direction context helpers.
  Validation: `cargo nextest run -p fret-ui-material3 --lib tab_keyboard_direction_maps_arrow_keys_by_layout_direction tab_indicator_fallback_position_mirrors_logical_index_in_rtl`; `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state rtl_tabs_arrow_left_moves_to_next_logical_tab_without_wrapping`.
  Review: DONE. Unit tests lock keyboard/indicator fallback math, and diagnostics integration
  verifies RTL ArrowLeft moves to the next logical tab without relying on wraparound.
  Handoff: Close after full gates.

## M2 - Closeout

- [x] M3TABRTL-090 [owner=codex] [deps=M3TABRTL-020] [scope=docs/workstreams/material3-tabs-rtl-indicator-v1]
  Goal: Close the lane with source-backed residuals.
  Validation: all lane gates pass.
  Review: DONE. The lane closes with focused tests and explicit residuals for full Flex RTL layout
  mirroring and selected-tab auto-scroll.
  Handoff: Start separate follow-ons for layout-engine RTL mirroring or scroll-selected-tab parity.
