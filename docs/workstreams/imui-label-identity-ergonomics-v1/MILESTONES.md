# ImUi Label Identity Ergonomics v1 - Milestones

Status: closed
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

Current progress:

- `M1_BUTTON_LABEL_IDENTITY_SLICE_2026-04-28.md` landed the private parser and its unit tests.

Exit criteria:

- parser semantics are explicit,
- unit tests cover the Dear ImGui-style marker cases,
- and the parser remains private until adoption proves the right surface.

## M2 - First Control Adoption

Current progress:

- The button family now hides `##` / `###` suffixes from painted labels and uses parsed identity
  keys for stable focus across visible-label changes and reorder.
- Selectable rows and menu item rows now share the same parser and have authoring proof coverage
  for suffix hiding and stable `###` identity across reorder.
- Checkbox, radio, switch, and slider now use parsed label identity as keyed helper-owned subtrees.
- Explicit-ID controls now keep their explicit IDs while stripping suffixes from visible labels:
  combo triggers, menu/submenu triggers, tab triggers, collapsing headers, and tree nodes.
- `separator_text` now strips suffixes from its rendered label.

Exit criteria:

- label-bearing controls in the first admitted set render visible labels without suffixes,
- explicit `a11y_label` and `test_id` behavior remains unchanged,
- and at least one stateful authoring proof demonstrates stable `###` identity across visible-label
  changes.

## M3 - Closeout

Result:

- `WORKSTREAM.json` moved to `closed`.
- `CLOSEOUT_AUDIT_2026-04-28.md` names adopted and deferred controls.
- `EVIDENCE_AND_GATES.md` records the final local gate set.
- Future runtime ID-stack debugging, table-header policy, localization, and `test_id` inference
  remain separate follow-on scope.
