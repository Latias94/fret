# View-Locals Authoring (Fearless Refactor v1)

Status: closed closeout lane
Last updated: 2026-03-20

Related:

- `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/TODO_LADDER_AUDIT_2026-03-20.md`
- `docs/workstreams/local-state-architecture-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- `docs/workstreams/local-state-facade-boundary-hardening-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- `docs/workstreams/action-write-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-17.md`
- `docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-20.md`
- `docs/authoring-golden-path-v2.md`
- `docs/examples/todo-app-golden-path.md`
- `CLOSEOUT_AUDIT_2026-03-20.md`

Closeout reading rule on 2026-03-20:

- treat this document as the historical design record for a now-closed narrow lane
- read the shipped outcome from `CLOSEOUT_AUDIT_2026-03-20.md`
- reopen this lane only if fresh cross-surface evidence shows that the shipped `1-2 inline / 3+ bundle`
  rule no longer covers the default app-lane pressure

## Why this workstream exists

The previous authoring lanes already closed several tempting but broader follow-ons:

- selector/query posture is closed,
- conversion-surface cleanup is closed,
- action-write helper growth is closed,
- and the `LocalState<T>` storage contract is explicitly not being reopened here.

What remains is narrower and more concrete:

> default app-lane examples still become noisier than necessary when one view owns several related
> `LocalState<T>` slots and keeps threading them through helpers or action bindings individually.

This lane exists to productize an organization style that already appears in real non-Todo proof
surfaces:

- `struct XxxLocals { ... }`
- `impl XxxLocals { fn new(cx: &mut AppUi<'_, '_>) -> Self }`
- optional `fn bind_actions(&self, cx: &mut AppUi<'_, '_>)`

The goal is not a new runtime abstraction. The goal is to make the default app-facing authoring
surface teach one steadier way to organize view-owned local handles once a render body grows past a
small inline example.

## Necessity note (2026-03-20)

This lane is justified by repeated cross-surface evidence, not by the Todo demos alone.

Current evidence cluster:

- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/src/simple_todo_demo.rs`
- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
- `apps/fret-cookbook/examples/form_basics.rs`
- `apps/fretboard/src/scaffold/templates.rs`
- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/src/liquid_glass_demo.rs`

What repeats across those surfaces:

- a render body declaring three or more related local slots,
- helper signatures that only exist to thread several `LocalState<T>` handles around,
- action-binding functions with long parameter lists,
- and first-party examples/templates teaching the wider version of the same pattern.

That is enough to open a dedicated narrow lane.

## Core judgment

The correct fix here is organizational, not infrastructural.

Do:

- standardize a tiny `*Locals` bundle pattern on first-party default app surfaces,
- teach `new(cx)` as the default construction point,
- allow `bind_actions(&self, cx)` when several typed actions share the same local bundle,
- and keep reads/writes explicit through the existing `LocalState<T>` and `cx.actions()` APIs.

Do not:

- add new runtime helper families,
- widen `fret::app::prelude::*`,
- or reopen the storage-model discussion just because render bodies are getting repetitive.

## Goals

1. Reduce default app-lane authoring noise when a view owns several related `LocalState<T>` slots.
2. Replace long helper parameter lists with a small view-owned bundle where the grouping is already
   semantically obvious.
3. Keep the shipped default explicit:
   - `LocalState<T>` remains the state primitive,
   - `cx.actions()` remains the action binding surface,
   - row payload writes still use the already-closed action-write posture.
4. Align examples, scaffold templates, docs, and source-policy tests on the same organization rule.
5. Keep the change applicable beyond Todo demos by proving it on at least one non-Todo example.

## Non-goals

- Adding new `cx.actions()` helpers, new `LocalState<T>` helpers, or new runtime sugar.
- Reopening selector/query/router scope.
- Reopening the long-term `LocalState<T>` storage contract.
- Forcing every example to create a locals bundle when one or two inline slots remain clearer.
- Moving shared-model/editor-grade surfaces to look identical to LocalState-first app surfaces.

## Proposed default rule

- Keep one or two trivial local slots inline when they are only used locally in a short render body.
- Prefer a small `*Locals` bundle once a view:
  - owns three or more related local slots, or
  - repeatedly passes those local handles into helpers/handlers, or
  - binds several actions over the same local handle set.
- Prefer `bind_actions(&self, cx)` only when the bundle already exists and the action group is
  clearly view-owned.

This is a productized organization rule for first-party authoring. It is not a new framework
capability.

## Initial proof surfaces

Canonical compare set:

- `apps/fret-examples/src/simple_todo_demo.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
- `apps/fretboard/src/scaffold/templates.rs`

Non-Todo proof:

- `apps/fret-cookbook/examples/form_basics.rs`

Docs/golden path:

- `docs/authoring-golden-path-v2.md`
- `docs/examples/todo-app-golden-path.md`
- `ecosystem/fret/README.md`

## Router scope note

Router stays out of scope.

The current pressure is about view-owned local handle organization inside one render body, not about
route state, history semantics, or nested route composition. Reopen router only if fresh evidence
shows the same organization problem on actual routed proof surfaces.
