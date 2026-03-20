# Selector / Query Authoring Density (Fearless Refactor v1) — Milestones

Status: active
Last updated: 2026-03-20

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `TODO.md`

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

## Milestone 3 — Lock docs and gates

Outcome:

- the chosen density reductions become the taught/default first-party posture

Deliverables:

- docs/templates/examples updated as needed
- source-policy/tests updated
