# Authoring Density Reduction (Fearless Refactor v1) — Milestones

Last updated: 2026-03-16

Related:

- Design: `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/DESIGN.md`
- Target interface state: `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- TODO: `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/TODO.md`
- Tracked-read audit: `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/TRACKED_READ_AUDIT_2026-03-16.md`
- Selector/query audit: `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/SELECTOR_QUERY_AUDIT_2026-03-16.md`
- Selector/query direction: `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/SELECTOR_QUERY_DIRECTION_2026-03-16.md`
- Child-collection audit: `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/CHILD_COLLECTION_AUDIT_2026-03-16.md`
- Closeout audit: `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- Authoring-surface closeout: `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- Action-first post-v1 summary: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`
- Into-element closeout target: `docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`

## Current status snapshot (as of 2026-03-16)

- **M0**: Met once this directory and the main docs indices land.
- **M1**: Closeout mode (shared tracked-read surface settled; remaining work is breadth cleanup and
  wording/gate retirement).
- **M2**: Met (query default-read cleanup is now landed on the default teaching surfaces, and the
  first LocalState-aware selector dependency bridge landed at the `fret` layer without prelude
  widening or `fret-selector` ownership creep).
- **M3**: Met (the post-M1/M2 child-collection audit found no new shared-surface gap; the
  canonical compare set and non-Todo app-facing proofs already converge on
  `ui::for_each_keyed(...)` + `ui::children![cx; ...]` + `ui::single(cx, child)`, while the
  remaining awkward seams are intentional advanced/interop boundaries).
- **M4**: Met (default docs/templates/examples now teach only the shorter path, source-policy
  gates protect that baseline, and the remaining older wording is explicitly classified as
  advanced/history-only context).

Overall reading:

- this is the next active post-v1 authoring refactor lane,
- it is not a reopen of the app/component/advanced taxonomy,
- and it is not a stealth runtime/state-architecture redesign.

## Current execution order

1. Freeze scope and evidence rules.
2. Audit repeated tracked-read ceremony and land the smallest justified shared reduction.
3. Audit selector/query ceremony and land the smallest justified shared reduction.
4. Re-measure keyed/list/default child-collection pressure after the read-side reductions.
5. Delete displaced public-looking wording and keep docs/templates/examples/gates aligned.

Tracked-read note on 2026-03-16:

- `TRACKED_READ_AUDIT_2026-03-16.md` now records that the first batch is primarily adoption cleanup:
  the shorter `AppUi` tracked-read path already existed, but first-party app surfaces had not
  migrated consistently.
- the likely next real shared-surface question has therefore narrowed to helper-heavy
  `UiCx` / `ElementContext` model-read ergonomics after that adoption cleanup is absorbed.
- second-pass proof across `simple_todo_demo`, `async_playground_demo`, and custom-effect/postprocess
  helpers shows repeated `cx.watch_model(&model).layout()/paint()` pressure outside Todo-only
  comparisons, so a narrow component-layer handle-first helper is justified there.
- third/fourth-pass proof across `genui_demo` and `imui_editor_proof_demo` shows that the
  unresolved larger pressure has now shifted from tracked-read helper shape to selector/query-style
  derived-state choreography.
- `genui_demo` has since absorbed its remaining low-risk tracked-read cleanup, so the remaining M1
  work is best treated as closeout-only breadth cleanup rather than another design phase.
- therefore M1 should be treated as design-complete with remaining breadth cleanup, and M2 should
  become the next active design step.
- `SELECTOR_QUERY_DIRECTION_2026-03-16.md` now narrows M2 further:
  - query read-side work starts as adoption/docs cleanup to the already-shipped handle-first read
    posture,
  - selector dependency choreography remains the first likely new shared-surface question,
  - and any LocalState-aware selector helper must land at the `fret` facade/runtime layer instead
    of teaching `fret-selector` about `LocalState<T>`.
- that first M2 batch is now implemented:
  - query examples/templates/docs teach `handle.layout(...).value_or_default()` instead of the
    older `value_or_else(QueryState::<T>::default)` spelling,
  - `fret::selector::{DepsBuilder, LocalDepsBuilderExt as _}` plus
    `LocalState::{watch_in, paint_in, layout_in, hit_test_in}` now provide the narrow
    LocalState-first selector path on helper-heavy `ElementContext` surfaces,
  - and the `fret` surface-policy tests now explicitly keep this lane out of
    `fret::app::prelude::*`.
- `CHILD_COLLECTION_AUDIT_2026-03-16.md` now closes the next question after M2:
  - the canonical compare set no longer teaches old single-child landing or keyed-list patterns,
  - `hello.rs` and `assets_reload_epoch_basics.rs` show the same child-collection posture outside
    Todo,
  - and the remaining `ViewElements` / `AnyElement` seams in advanced cookbook surfaces are now
    explicitly classified as intentional retained/interop boundaries rather than missing default
    authoring API.
- `CLOSEOUT_AUDIT_2026-03-16.md` now closes the workstream:
  - `docs/first-hour.md`, `docs/examples/todo-app-golden-path.md`,
    `docs/authoring-golden-path-v2.md`, and the scaffold/source-policy gates all agree on the
    shorter default path,
  - onboarding docs no longer teach `cx.watch_model(...)` or explicit
    `.into_element(cx)` / `AnyElement` seams as the default app-authoring posture,
  - and the remaining longer wording is now recorded as advanced/component/runtime or historical
    workstream context.

## Milestone 0 — Freeze the lane

Outcome:

- Maintainers agree on what this lane owns and what it does not own.

Deliverables:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `TODO.md`
- roadmap/docs index updates that point to this lane explicitly

Exit criteria:

- reviewers can tell that this is the next active authoring lane without reopening
  app/component/advanced or into-element design debates.

## Milestone 1 — Shorten tracked reads

Outcome:

- high-frequency tracked reads on the default path become materially shorter.

Deliverables:

- audited before/after evidence on the canonical compare set
- at least one non-todo proof surface
- one shorter default read story that still keeps invalidation intent explicit

Exit criteria:

- default-path docs/examples/templates no longer teach the previous longer read plumbing as the
  primary story,
- the new read story is still explicit enough that maintainers can reason about invalidation.

## Milestone 2 — Shorten LocalState-first selector/query authoring

Outcome:

- derived/async state feels like an extension of the same app-facing dialect instead of a jump to a
  more internal-looking style.

Deliverables:

- selector dependency/read reduction for view-owned LocalState-first examples
- query observe/read reduction for default app-facing examples
- written layering rationale for why query cleanup and selector dependency reduction take different
  forms
- proof that the resulting surface still keeps read-vs-write ownership explicit

Exit criteria:

- the third-rung `todo` surface is materially shorter,
- at least one additional non-todo surface benefits from the same reduction,
- the solution does not widen the default app prelude,
- default docs/templates stop teaching `clone_model()` as the first-contact selector dependency
  story on LocalState-first app surfaces,
- and query examples stop teaching older `watch(...).layout().value_or_else(...)`-style defaults
  where the shipped handle-first `value_or_default()` path already suffices.

## Milestone 3 — Re-evaluate keyed/list/default child-collection pressure

Outcome:

- the repo decides whether any remaining list/collection noise is still a real shared-surface
  problem after the read-side reductions.

Deliverables:

- an audit pass across the canonical compare set plus at least one non-todo surface
- either:
  - a "docs/adoption only" verdict, or
  - one narrow justified shared helper/change

Exit criteria:

- maintainers can point to evidence for why list/collection pressure does or does not justify new
  public API.

## Milestone 4 — Delete the displaced path and lock the gates

Outcome:

- the shorter default path is the only taught default path.

Deliverables:

- default docs/templates/examples updated
- stale public-looking wording removed from the taught path
- source-policy/tests/gates updated for the new baseline

Exit criteria:

- the repo does not teach two co-equal default paths,
- the remaining longer wording is either gone or clearly marked as advanced/history-only.
