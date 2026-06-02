# Material3 Tabs Divider v1 - TODO

Status: Closed
Last updated: 2026-05-30

Task IDs use `M3TABDIV-*`.

## M0 - Lane Setup

- [x] M3TABDIV-010 [owner=codex] [deps=none] [scope=docs/workstreams/material3-tabs-divider-v1]
  Goal: Split TabRow divider rendering into a narrow follow-on.
  Validation: `python -m json.tool docs/workstreams/material3-tabs-divider-v1/WORKSTREAM.json | Out-Null`; `python tools/check_workstream_catalog.py`.
  Review: DONE. The lane records Compose `HorizontalDivider()` defaults and Material Web
  navigation-tab divider tokens as the source stack.
  Handoff: Start M3TABDIV-020.

## M1 - Divider Layer

- [x] M3TABDIV-020 [owner=codex] [deps=M3TABDIV-010] [scope=ecosystem/fret-ui-material3/src/{tabs.rs,tokens/tabs.rs,tokens/v30.rs},ecosystem/fret-ui-material3/tests/{tabs_state.rs,automation_surface.rs}]
  Goal: Render the bottom divider as a TabRow layer, expose a stable part test id, and route tokens.
  Validation: `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state`; `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_tabs_exposes_stable_part_test_ids`; `cargo nextest run -p fret-ui-material3 --lib tokens::v30`.
  Review: DONE. `m3-tabs.divider` is live, the divider spans the row bottom, and focused tests
  verify the active indicator shares its bottom edge.
  Handoff: Close after full gates.

## M2 - Closeout

- [x] M3TABDIV-090 [owner=codex] [deps=M3TABDIV-020] [scope=docs/workstreams/material3-tabs-divider-v1]
  Goal: Close the lane with source-backed residuals.
  Validation: all lane gates pass.
  Review: DONE. The lane closes with tests and workstream evidence.
  Handoff: Future work should start separate follow-ons for public divider customization,
  RTL indicator positioning, selected-tab auto-scroll, or gallery snippets.
