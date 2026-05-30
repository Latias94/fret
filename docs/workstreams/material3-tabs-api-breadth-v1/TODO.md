# Material3 Tabs API Breadth v1 - TODO

Status: Active
Last updated: 2026-05-30

Task IDs use `M3TAB-*`.

## M0 - Lane Setup

- [x] M3TAB-010 [owner=codex] [deps=none] [scope=docs/workstreams/material3-tabs-api-breadth-v1]
  Goal: Open the durable Tabs API breadth lane with source-backed primary/secondary facts and
  gates.
  Validation: `python -m json.tool docs/workstreams/material3-tabs-api-breadth-v1/WORKSTREAM.json | Out-Null`; `python tools/check_workstream_catalog.py`.
  Review: DONE. The lane records Compose as the secondary tabs source and Material Web v30 as the
  primary generated-token source.
  Handoff: Start M3TAB-020 with the public variant API and focused geometry tests.

## M1 - Secondary Variant Slice

- [x] M3TAB-020 [owner=codex] [deps=M3TAB-010] [scope=ecosystem/fret-ui-material3/src/{foundation/active_indicator.rs,tabs.rs,tokens/tabs.rs,tokens/v30.rs},ecosystem/fret-ui-material3/tests/tabs_state.rs]
  Goal: Add an explicit secondary Tabs variant, route typed token access by variant, and prove
  secondary fixed/scrollable active indicators use full tab width while primary keeps content-sized
  behavior.
  Validation: `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state`; `cargo nextest run -p fret-ui-material3 --lib tokens::v30`; `cargo check -p fret-ui-material3 --features diagnostics --tests`.
  Review: DONE. `TabsVariant::Secondary` is public and opt-in, primary remains the default, typed
  token access now routes by primary/secondary variant, v30 seeds Compose-backed secondary tab
  aliases, and focused tests prove fixed/scrollable secondary indicators use full tab width while
  primary keeps content-sized geometry. The shared active-indicator canvas now applies target-sized
  minimum bounds so scrollable indicator rects cannot clamp to zero while parent width is still
  resolving.
  Handoff: Decide whether to close with this API/geometry evidence or add optional secondary token
  fixture rows in M3TAB-030.

- [ ] M3TAB-030 [owner=codex] [deps=M3TAB-020] [scope=ecosystem/fret-ui-material3/src/tokens,ecosystem/fret-ui-material3/tests/fixtures,docs/workstreams/material3-tabs-api-breadth-v1]
  Goal: Add secondary tabs token fixture coverage only if M3TAB-020 introduces secondary literal
  tokens that are not already protected by v30 and geometry gates.
  Validation: `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures`.
  Review: Pending.
  Handoff: Keep this skipped if it would duplicate M3TVM token-matrix evidence without new API
  breadth value.

## M2 - Closeout

- [ ] M3TAB-090 [owner=codex] [deps=M3TAB-020] [scope=docs/workstreams/material3-tabs-api-breadth-v1]
  Goal: Close the lane or split richer tab breadth into follow-ons.
  Validation: all lane gates pass or residuals are source-backed and split.
  Review: Pending.
  Handoff: Candidate residuals are icon-and-label tabs, divider rendering, and gallery snippets.
