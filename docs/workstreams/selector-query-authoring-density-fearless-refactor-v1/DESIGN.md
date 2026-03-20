# Selector / Query Authoring Density (Fearless Refactor v1)

Status: closed closeout lane (query projections landed; selector no-new-API verdict)
Last updated: 2026-03-20

Related:

- `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/TODO_LADDER_AUDIT_2026-03-20.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/action-write-surface-fearless-refactor-v1/DESIGN.md`
- `docs/authoring-golden-path-v2.md`
- `docs/examples/todo-app-golden-path.md`
- `CLOSEOUT_AUDIT_2026-03-20.md`

Closeout reading rule on 2026-03-20:

- treat this document as the historical design record for a now-closed narrow density lane
- read the shipped outcome from `CLOSEOUT_AUDIT_2026-03-20.md`
- reopen this lane only if fresh cross-surface evidence shows:
  - repeated query semantic pressure not already covered by the shipped projections, or
  - repeated selector borrowed-input pressure beyond the Todo scaffold

## Why this workstream exists

The earlier lanes already closed the ownership and default-path boundary questions:

- `dataflow-authoring-surface-fearless-refactor-v1` closed the selector/query default posture,
- `authoring-density-reduction-fearless-refactor-v1` closed the broader default-path helper pass,
- `action-write-surface-fearless-refactor-v1` closed the write-budget question.

What remains is narrower:

> selector/query on the default app lane is now correctly layered, but still denser than it should
> be on several first-party surfaces.

This lane exists to reduce that remaining density without reopening:

- prelude widening,
- router scope,
- raw shared-model/editor-grade explicitness,
- or query engine lifecycle semantics.

## Necessity note (2026-03-20)

This lane is justified by repeated non-Todo evidence, not by the `todo` scaffold alone.

Current evidence cluster:

- `apps/fretboard/src/scaffold/templates.rs`
- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`
- `apps/fret-examples/src/async_playground_demo.rs`

What repeats across those surfaces:

- explicit query status labeling,
- explicit loading-vs-refreshing branching,
- repeated `error.is_some()` / `data.is_some()` conditionals,
- and, separately, selector compute closures that often need owned intermediate structs because the
  current LocalState-first selector lane computes from cloned values.

This is enough to open a new lane.

## Problem split

### Query-side density

The query lane is now correctly honest about:

- `QueryKey`
- `QueryPolicy`
- `cx.data().query*`
- `handle.read_layout(cx)`

But app code still repeatedly rebuilds the same semantic checks:

- `"Idle" / "Loading" / "Success" / "Error"` labels,
- `Loading && data.is_some()` as "refreshing",
- `error.is_some()` and `data.is_some()` conditionals.

The first batch here should favor semantically honest projection helpers over new DSL.

### Selector-side density

The selector lane is correctly narrowed to:

- `cx.data().selector_layout(inputs, compute)` for LocalState-first app surfaces,
- explicit raw selector signatures for shared-model/editor-grade work.

But the current LocalState-first selector input machinery still computes from cloned values.
That keeps the surface correct, but it may force app code into extra owned `Derived` / `Snapshot`
shaping even when the real need is only a borrowed projection.

This selector side needs a more careful audit before code changes.

## Goals

1. Reduce selector/query density on the default app lane without widening the app prelude.
2. Keep query key/policy/fetch and lifecycle semantics explicit.
3. Keep selector invalidation intent explicit.
4. Keep router out of scope unless fresh cross-surface evidence proves the same density problem.
5. Land code, docs, and gates together for each justified batch.

## Non-goals

- Reopening the closed selector/query ownership decision from the dataflow lane.
- Adding Todo-only helpers.
- Hiding query lifecycle behind recipe-like "success-only" DSL.
- Moving reusable ecosystem crates onto `fret`.
- Making editor-grade shared-model surfaces look identical to LocalState-first app surfaces.

## Historical execution direction

### Batch 1

Land no-regret query-state projection helpers:

- status-to-text,
- status predicates,
- refreshing detection.

Target:

- reduce repeated branch noise on `query_demo`, `query_async_tokio_demo`, `query_basics`, and
  related app-facing proof surfaces,
- without changing create-side query semantics.

### Batch 2

Audit whether selector density truly needs a borrowed-compute follow-on on the LocalState-first
lane.

Promotion rule:

- only continue once the same pressure is proven on at least one non-Todo surface beyond the
  scaffold.
