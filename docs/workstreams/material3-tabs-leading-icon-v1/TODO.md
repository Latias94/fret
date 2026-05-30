# Material3 Tabs Leading Icon v1 - TODO

Status: Closed
Last updated: 2026-05-30

Task IDs use `M3TABLI-*`.

## M0 - Lane Setup

- [x] M3TABLI-010 [owner=codex] [deps=none] [scope=docs/workstreams/material3-tabs-leading-icon-v1]
  Goal: Open a narrow follow-on for Compose `LeadingIconTab` parity.
  Validation: `python -m json.tool docs/workstreams/material3-tabs-leading-icon-v1/WORKSTREAM.json | Out-Null`; `python tools/check_workstream_catalog.py`.
  Review: DONE. The lane records `LeadingIconTab` as the target and keeps stacked 72dp tabs out of
  scope.
  Handoff: Start the API/token slice.

## M1 - API, Tokens, Layout

- [x] M3TABLI-020 [owner=codex] [deps=M3TABLI-010] [scope=ecosystem/fret-ui-material3/src/{tabs.rs,tokens/tabs.rs,tokens/v30.rs},ecosystem/fret-ui-material3/tests/tabs_state.rs]
  Goal: Add a leading icon API to `TabItem`, render icon + label content with Material spacing, and
  route primary/secondary icon tokens.
  Validation: `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state`; `cargo nextest run -p fret-ui-material3 --lib tokens::v30`.
  Review: DONE. `TabItem::leading_icon(IconId)` is public, icon size/color are token-routed, v30
  seeds secondary icon aliases, and tests cover 24px icon size, 8px icon-label gap, and primary
  content-sized indicator geometry.
  Handoff: Close the lane unless stacked icon + label tabs are promoted into a separate follow-on.

## M2 - Closeout

- [x] M3TABLI-090 [owner=codex] [deps=M3TABLI-020] [scope=docs/workstreams/material3-tabs-leading-icon-v1]
  Goal: Close the follow-on with focused evidence and residuals split.
  Validation: all lane gates pass.
  Review: DONE. This lane closes as the leading-icon slice only.
  Handoff: Future work should start a new lane for stacked 72dp icon + label tabs, gallery snippets,
  divider rendering, RTL indicator positioning, or selected-tab auto-scroll.
