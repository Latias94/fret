# Direction Infrastructure v1 - TODO

Status: Closed
Last updated: 2026-05-30

Task IDs use `DIRINFRA-*`.

## M0 - Lane Setup

- [x] DIRINFRA-010 [owner=codex] [deps=none] [scope=docs/workstreams/direction-infrastructure-v1]
  Goal: Define the lane boundary for direction infrastructure.
  Validation: `python -m json.tool docs/workstreams/direction-infrastructure-v1/WORKSTREAM.json | Out-Null`; `python tools/check_workstream_catalog.py`.
  Review: DONE. The first slice is shared policy extraction, not global Flex RTL layout.
  Handoff: Start DIRINFRA-020.

## M1 - Shared Direction Policy

- [x] DIRINFRA-020 [owner=codex] [deps=DIRINFRA-010] [scope=ecosystem/fret-ui-kit/src/primitives/{direction.rs,roving_focus_group.rs},ecosystem/fret-ui-shadcn/src/rtl.rs,ecosystem/fret-ui-material3/src/{chip_set.rs,segmented_button.rs,tabs.rs}]
  Goal: Promote duplicated horizontal RTL key semantics and visual index helpers into
  `fret-ui-kit::primitives::direction`, then migrate representative callsites.
  Validation: focused kit, shadcn, and Material3 tests.
  Review: DONE. `fret-ui-kit::primitives::direction` now owns the shared horizontal arrow
  semantics and visual position helper; kit roving focus, shadcn RTL helpers, and Material3
  ChipSet/SegmentedButton/Tabs reuse it.
  Handoff: Start a separate mechanism-layer follow-on for Flex RTL physical placement.

## M2 - Closeout

- [x] DIRINFRA-090 [owner=codex] [deps=DIRINFRA-020] [scope=docs/workstreams/direction-infrastructure-v1]
  Goal: Record evidence, residuals, and the next mechanism-layer follow-on.
  Validation: all lane gates pass.
  Review: DONE. The closeout explicitly leaves global Flex RTL layout outside this lane.
  Handoff: Next lane should own layout-engine direction input and physical row placement.
