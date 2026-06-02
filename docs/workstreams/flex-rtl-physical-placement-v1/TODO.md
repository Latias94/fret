# Flex RTL Physical Placement v1 - TODO

Status: Closed
Last updated: 2026-05-30

Task IDs use `FLEXRTL-*`.

## M0 - Lane Setup

- [x] FLEXRTL-010 [owner=codex] [deps=none] [scope=docs/workstreams/flex-rtl-physical-placement-v1]
  Goal: Define the mechanism-layer boundary for Flex RTL physical placement.
  Validation: `python -m json.tool docs/workstreams/flex-rtl-physical-placement-v1/WORKSTREAM.json | Out-Null`.
  Review: DONE. The lane captures provider direction on elements rather than expanding `FlexProps`.
  Handoff: Start FLEXRTL-020.

## M1 - Direction Snapshot And Layout Bridge

- [x] FLEXRTL-020 [owner=codex] [deps=FLEXRTL-010] [scope=crates/fret-ui/src/{element.rs,elements/cx.rs,declarative,layout,tree}]
  Goal: Store `LayoutDirection` on element/frame records and route horizontal Flex rows through
  `RowReverse` under RTL.
  Validation: focused `fret-ui` layout tests.
  Review: DONE. `AnyElement` now captures provider direction, `ElementRecord` exposes it to layout,
  and horizontal RTL Flex rows use `RowReverse`.
  Handoff: Start FLEXRTL-030.

## M2 - Evidence And Closeout

- [x] FLEXRTL-030 [owner=codex] [deps=FLEXRTL-020] [scope=docs/workstreams/flex-rtl-physical-placement-v1]
  Goal: Record evidence, residuals, and verification gates.
  Validation: formatting, focused tests, layering, workstream catalog.
  Review: DONE. Closeout records shipped scope, gates, and residual logical-edge follow-ons.
  Handoff: Closed.
