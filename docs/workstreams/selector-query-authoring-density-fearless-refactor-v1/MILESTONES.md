# Selector / Query Authoring Density (Fearless Refactor v1) — Milestones

Status: Closed closeout lane (query projections landed; selector no-new-API verdict)
Last updated: 2026-03-20

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `TODO.md`
- `SELECTOR_BORROWED_INPUT_AUDIT_2026-03-20.md`
- `CLOSEOUT_AUDIT_2026-03-20.md`

## Current execution stance (2026-03-20)

- Milestone 1 is landed:
  the shipped query helper batch now covers semantic status text/predicates and refreshing/error
  presence checks without hiding key/policy/fetch ownership.
- Milestone 2 is landed on a no-new-API selector verdict:
  the Todo scaffold remains the strongest pressure point, but the audited non-Todo surfaces do not
  justify reopening the public selector surface.
- Milestone 3 is landed:
  docs, scaffold guidance, first-party proof surfaces, and gates now point at the same shipped
  selector/query posture.
- Read `CLOSEOUT_AUDIT_2026-03-20.md` as the final interpretation of this lane.

## Milestone 0 — Freeze scope

Outcome:

- maintainers can explain why this lane is new even though selector/query ownership already closed

Exit criteria:

- docs/indices point to this lane
- the query-vs-selector split is explicit
- router is explicitly adjacent-only

## Milestone 1 — Query semantic projection helpers

Outcome:

- common query semantic checks become shorter without hiding lifecycle semantics

Deliverables:

- a small `QueryStatus` / `QueryState<T>` projection batch
- adoption on at least two app-facing proof surfaces
- tests for the new semantic helpers

Exit criteria:

- first-party query surfaces stop rebuilding the same label/refreshing checks manually
- key/policy/fetch remains explicit

## Milestone 2 — Selector borrowed-input audit

Outcome:

- maintainers decide whether the selector side has a real LocalState-first borrowed-projection gap

Deliverables:

- an audit note over Todo plus at least one non-Todo app-facing surface
- either:
  - a no-new-API verdict, or
  - one narrow borrowed-compute direction

Current 2026-03-20 reading:

- this milestone is currently met on a no-new-API verdict;
- the Todo scaffold remains the strongest pressure point, but the rest of the audited app-facing
  selector sites do not yet justify reopening the public selector surface.

## Milestone 3 — Lock docs and gates

Outcome:

- the chosen density reductions become the taught/default first-party posture

Deliverables:

- docs/templates/examples updated as needed
- source-policy/tests updated

Closeout note on 2026-03-20:

- this milestone is now satisfied;
- no further selector/query helper growth is queued inside this lane.
