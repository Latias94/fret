# Material3 Tabs Stacked Icon v1 - TODO

Status: Closed
Last updated: 2026-05-30

Task IDs use `M3TABSI-*`.

## M0 - Lane Setup

- [x] M3TABSI-010 [owner=codex] [deps=none] [scope=docs/workstreams/material3-tabs-stacked-icon-v1]
  Goal: Split the stacked icon + label tab shape into a narrow follow-on instead of reopening the
  closed leading-icon lane.
  Validation: `python -m json.tool docs/workstreams/material3-tabs-stacked-icon-v1/WORKSTREAM.json | Out-Null`; `python tools/check_workstream_catalog.py`.
  Review: DONE. The lane records Compose `TabBaselineLayout` as the runtime layout truth and
  Material Web 64px as a documented source divergence.
  Handoff: Start M3TABSI-020.

## M1 - API and Layout

- [x] M3TABSI-020 [owner=codex] [deps=M3TABSI-010] [scope=ecosystem/fret-ui-material3/src/{tabs.rs,tokens/tabs.rs,tokens/v30.rs},ecosystem/fret-ui-material3/tests/tabs_state.rs]
  Goal: Add a placement-aware tab icon model, expose stacked icon API, and raise stacked rows to
  72px.
  Validation: `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state`; `cargo nextest run -p fret-ui-material3 --lib tokens::v30`.
  Review: DONE. `TabIconPlacement`, `TabItem::icon(...)`, and `TabItem::stacked_icon(...)` shipped.
  Stacked tabs use vertical icon-over-label content and promote the row height to the Compose 72px
  large height.
  Handoff: Close after full gates.

## M2 - Closeout

- [x] M3TABSI-090 [owner=codex] [deps=M3TABSI-020] [scope=docs/workstreams/material3-tabs-stacked-icon-v1]
  Goal: Close the lane with tests and source-backed residuals.
  Validation: all lane gates pass.
  Review: DONE. The lane closes with focused geometry and token gates.
  Handoff: Future work should start separate follow-ons for exact text-baseline placement, divider
  rendering, RTL indicator positioning, selected-tab auto-scroll, or gallery snippets.
