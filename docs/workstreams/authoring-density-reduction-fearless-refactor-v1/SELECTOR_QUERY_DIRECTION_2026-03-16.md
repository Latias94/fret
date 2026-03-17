# Selector / Query Direction — 2026-03-16

Status: Historical direction note; query portion superseded on 2026-03-17
Last updated: 2026-03-17

Related:

- `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/TODO.md`
- `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/SELECTOR_QUERY_AUDIT_2026-03-16.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/QUERY_READ_SURFACE_CLOSEOUT_2026-03-17.md`

## Supersession note (2026-03-17)

- The query portion of this note is historical only.
- The current shipped default app-lane query read posture is `handle.read_layout(cx)`.
- Keep this file for the original M2 audit context and for the selector ownership rationale, not as
  the current query API direction.
- The experimental named `LocalDepsBuilderExt` selector export also became historical on
  2026-03-17.
- The shipped selector posture keeps `cx.data().selector_layout(...)` as the LocalState-first
  default path, while the explicit `fret::selector` lane retains only raw selector nouns such as
  `DepsBuilder` / `DepsSignature` / `Selector`.

## Why this note exists

Milestone 1 already settled the shared tracked-read helper question.

Milestone 2 must not reopen that decision by treating every remaining `selector` / `query`
friction point as proof that another broad read-side API family is needed.

This note records the narrower split:

- which M2 pressure is still mostly docs/adoption drift,
- which pressure is a real app-facing surface gap,
- and which layering constraints should govern the next refactor.

## Decision summary

- Historical starting point: query read-side pressure first looked like **adoption/docs cleanup**,
  not a new shared API request.
- LocalState-first selector dependency choreography is the first likely **real shared-surface gap**
  in M2.
- M2 should therefore start by:
  - normalizing taught query reads to the already-shipped shorter handle-first posture, and
  - designing the smallest `LocalState`-aware selector dependency bridge at the `fret`
    app-facing layer.

## Evidence cluster A — Query read-side is mostly adoption drift

Representative evidence:

- `apps/fret-examples/src/query_demo.rs:71`
- `apps/fret-examples/src/query_async_tokio_demo.rs:100`
- `apps/fret-cookbook/examples/query_basics.rs:91`
- `apps/fret-examples/src/async_playground_demo.rs:857`
- `apps/fret-examples/src/markdown_demo.rs:486`
- `docs/examples/todo-app-golden-path.md:331`
- `apps/fretboard/src/scaffold/templates.rs:445`
- `ecosystem/fret/src/view.rs:437`

Observed pattern:

- a handle is produced by `cx.data().query(...)` / `query_async(...)`,
- the handle is then read via `layout(cx)` or `layout_query(cx)`,
- and many call sites still spell the default fallback as
  `value_or_else(QueryState::<T>::default)`.

What this means:

- the app-facing handle-first read path already existed through `TrackedStateExt` on
  `QueryHandle<T>` (`handle.layout(cx)` on the `fret` app surface at the time of this note),
- the shipped default app posture later narrowed that teaching surface to `handle.read_layout(cx)`,
- the declarative/component path already has the equivalent `layout_query(cx)` helper,
- the repo still teaches or uses older/staler spellings in multiple places.

What this does **not** prove yet:

- it does not prove that Fret needs a new `query_state(cx)` or `when_success(...)` family,
- because much of the repeated branching after the read is the real query lifecycle
  (`status/data/error/retry`) rather than accidental plumbing.

Immediate rule for M2:

- historical 2026-03-16 cleanup target: shorten default fallback reads before inventing more
  shared query sugar,
- superseded 2026-03-17 shipped posture: teach `handle.read_layout(cx)` on the app path when the
  default fallback is `QueryState::<T>::default()`,
- keep `handle.layout_query(cx).value_or_default()` on declarative/component paths,
- only revisit shared query sugar after the repo has re-measured the post-cleanup surface.

## Evidence cluster B — Selector dependency choreography is a real surface gap

Representative evidence:

- `apps/fretboard/src/scaffold/templates.rs:403`
- `docs/examples/todo-app-golden-path.md:297`
- `apps/fret-examples/src/markdown_demo.rs:605`
- `ecosystem/fret/src/view.rs:389`
- `ecosystem/fret-selector/src/ui.rs:67`
- `ecosystem/fret/src/lib.rs:565`

Observed pattern:

- first-contact LocalState-first examples still bounce through `clone_model()`,
- then build dependency signatures with `DepsBuilder::model_rev(&model)`,
- then read the same values again through raw `cx.watch_model(...)`,
- or, in more advanced examples, fall all the way down to
  `observe_model(...) + models().revision(...) + models().read(...)`.

What this means:

- the pain is not "selectors need a shorter compute read helper";
  `LocalState` already has the shorter read-side posture on the app lane
  (`state.layout(cx).value_or_default()`, `state.paint(cx)...`, etc.),
- the pain is that dependency registration for LocalState-first selectors still teaches raw
  model-handle choreography before the author actually needs shared-model ownership.

That makes this a real M2 surface question:

- not because selectors should become magical,
- but because the default app path should not require ordinary authors to escape into `Model<T>`
  just to express view-owned derived state.

## Ownership and layering decision

The next refactor must respect these boundaries:

- `fret-selector` stays the portable selector crate and should not learn about `LocalState<T>`.
- `fret::app::prelude::*` stays frozen; M2 is not allowed to solve density by widening imports.
- any LocalState-aware selector sugar belongs in `ecosystem/fret` (the app-facing facade/runtime
  layer), not in `fret-selector` core.

Why this matters:

- `LocalState<T>` lives in `ecosystem/fret`,
- `fret-selector` currently depends only on `fret-runtime` and `fret-ui`,
- making `fret-selector` depend on `LocalState` would invert the crate layering for a default-path
  convenience problem.

## Candidate implementation direction

### Query lane

Start with cleanup, not invention:

1. update docs/templates/examples to the existing shorter handle-first reads,
2. prefer `value_or_default()` wherever the fallback is `QueryState::<T>::default()`,
3. re-measure whether the remaining query pain is really read-side plumbing or just normal
   lifecycle handling.

Until that cleanup lands, M2 should treat new shared query helper proposals as unproven.

### Selector lane

Bias toward the smallest bridge that removes raw-model escape hatches from LocalState-first
examples while keeping invalidation explicit.

Target property, not final spelling:

- a `LocalState<T>` should be able to contribute an observed dependency token/revision for
  `layout` / `paint` / `hit_test` without forcing app authors to clone the underlying
  `Model<T>` first.

That keeps the selector story explicit:

- deps closure still declares what invalidates recomputation,
- compute closure still owns the actual derived-state calculation,
- raw `Model<T>` choreography remains available for advanced/shared-ownership surfaces.

## Explicit non-directions

M2 should explicitly avoid:

- a `query_state(cx)` / `query_result(cx)` family before the query cleanup pass is absorbed,
- a broad `selector_local(...)` / `selector2(...)` family explosion,
- prelude widening,
- redesigning selector/query runtime ownership,
- forcing advanced shared-model surfaces to look identical to LocalState-first app surfaces.

## M2 exit criteria from this note

The design phase for M2 is complete when maintainers can point to all three statements as settled:

1. query read-side work starts as docs/adoption cleanup against existing helpers,
2. selector work starts with a narrow LocalState-aware dependency bridge at the `fret` layer,
3. neither change reopens M1 tracked-read design or widens the default app prelude.
