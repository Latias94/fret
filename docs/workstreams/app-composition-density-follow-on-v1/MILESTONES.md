# App Composition Density Follow-on v1 — Milestones

Status: closeout tracker
Last updated: 2026-03-17

Related:

- Design: `docs/workstreams/app-composition-density-follow-on-v1/DESIGN.md`
- Target interface state: `docs/workstreams/app-composition-density-follow-on-v1/TARGET_INTERFACE_STATE.md`
- TODO: `docs/workstreams/app-composition-density-follow-on-v1/TODO.md`
- App-shell composition audit: `docs/workstreams/app-composition-density-follow-on-v1/APP_SHELL_COMPOSITION_AUDIT_2026-03-17.md`
- Query invalidation audit: `docs/workstreams/app-composition-density-follow-on-v1/QUERY_INVALIDATION_SHELL_AUDIT_2026-03-17.md`
- Todo golden path: `docs/examples/todo-app-golden-path.md`
- Query basics: `apps/fret-cookbook/examples/query_basics.rs`
- Query demo: `apps/fret-examples/src/query_demo.rs`
- Query async demo: `apps/fret-examples/src/query_async_tokio_demo.rs`
- Router boundary check: `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/ECOSYSTEM_ADAPTATION_AND_ROUTER_AUDIT_2026-03-17.md`

## Current status snapshot (as of 2026-03-17)

- **M0**: Met once this directory and the main docs indices land.
- **M1**: Met. The audit found no new shared framework helper gap: cookbook already ships the
  reusable first-party page-shell answer, and the remaining duplication is example/scaffold
  discipline rather than `fret` facade surface pressure. See
  `APP_SHELL_COMPOSITION_AUDIT_2026-03-17.md`.
- **M2**: Met. The query invalidation shell is now grouped on `cx.data().invalidate_query(...)` /
  `cx.data().invalidate_query_namespace(...)` for `AppUi` / extracted `UiCx`, with raw
  `with_query_client(...)` retained for pure app/driver boundaries. See
  `QUERY_INVALIDATION_SHELL_AUDIT_2026-03-17.md`.
- **M3**: Pending. If a narrower posture lands, docs/examples/templates/gates must delete the
  displaced first-contact wording together.

Overall reading:

- this started as the only open authoring follow-on after the read/write closeout lanes,
- it stayed narrower than the earlier density lane,
- M1 is now closed on a no-new-API verdict,
- M2 is now closed on the grouped query invalidation shell,
- and the remaining open question is only the final M3 delete/lock discipline.

## Milestone 0 — Freeze the lane

Outcome:

- Maintainers can tell exactly what this lane owns.

Deliverables:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `TODO.md`
- roadmap/docs index updates that point to this lane

Exit criteria:

- reviewers can see that this lane owns only:
  - default app-shell composition density,
  - default app-lane query invalidation shell,
  - and the related docs/gates cleanup
- and does **not** own router, selector/query read redesign, or write-side surface growth.

## Milestone 1 — Audit and reduce app-shell composition density

Outcome:

- the repo decides whether the remaining page/root wrapper noise is:
  - docs/adoption drift,
  - a first-party helper-discipline problem,
  - or one narrow shared helper gap.

Deliverables:

- an audit across the primary detection surface
- at least one non-Todo proof surface
- a written owner decision for any change that survives the audit

Exit criteria:

- maintainers can point to concrete repeated wrapper patterns,
- the chosen reduction stays on the app-facing lane,
- and router/component/advanced surfaces are not pulled into the same fix.

## Milestone 2 — Audit and reduce query invalidation shell

Outcome:

- default app-lane query invalidation either gets one grouped story or is explicitly left raw for
  a documented reason.

Deliverables:

- an audit of repeated `with_query_client(...)` + redraw shell usage
- evidence from Todo-adjacent query examples plus at least one non-Todo app surface
- a layering note explaining why any grouped helper belongs in the app-facing layer rather than in
  `fret-query`
- migrations for first-party docs/examples if a grouped helper lands

Exit criteria:

- the repo no longer teaches raw query-client plumbing as the only first-contact app recipe unless
  the audit explicitly decides that keeping it raw is the correct product choice,
- and the chosen story still keeps query ownership explicit.

## Milestone 3 — Delete the displaced wording and lock the gates

Outcome:

- the narrower story becomes the only taught default story.

Deliverables:

- default docs/examples/templates updated
- source-policy/tests/gates refreshed
- stale first-contact wording removed from the taught path

Exit criteria:

- the repo does not teach two co-equal app-lane stories for the same pattern,
- and router/default dataflow docs remain unchanged except for boundary notes.
