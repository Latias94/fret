# ImUi Label Identity Ergonomics v1 - Milestones

Status: active execution lane
Last updated: 2026-04-28

## M0 - Baseline and Tracking

Exit criteria:

- the lane is listed in repo-level indexes,
- the older identity/geometry lanes stay closed,
- and the initial hygiene gates pass.

Evidence:

- `docs/workstreams/imui-control-geometry-stability-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-04-19.md`
- `ecosystem/fret-imui/src/frontend.rs`

## M1 - Parser Contract

Exit criteria:

- parser semantics are explicit,
- unit tests cover the Dear ImGui-style marker cases,
- and the parser remains private until adoption proves the right surface.

## M2 - First Control Adoption

Exit criteria:

- label-bearing controls in the first admitted set render visible labels without suffixes,
- explicit `a11y_label` and `test_id` behavior remains unchanged,
- and at least one stateful authoring proof demonstrates stable `###` identity across visible-label
  changes.

## M3 - Closeout

Exit criteria:

- `WORKSTREAM.json` moves to `closed`,
- adopted and deferred controls are named,
- final gates are recorded,
- and future identity debugging/tooling scope is split into narrower follow-ons.
