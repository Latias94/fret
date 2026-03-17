# Default Todo Authoring Status — 2026-03-17

Status: evidence note
Last updated: 2026-03-17

Related:

- `DESIGN.md`
- `MILESTONES.md`
- `TODO.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/authoring-golden-path-v2.md`
- `apps/fretboard/src/scaffold/templates.rs`
- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `docs/workstreams/action-write-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-17.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/QUERY_READ_SURFACE_CLOSEOUT_2026-03-17.md`
- `QUERY_INVALIDATION_SHELL_AUDIT_2026-03-17.md`

## Why this note exists

The app-composition follow-on is now a closeout / maintenance lane, but maintainers still need one
short answer to a practical question:

- how good does writing the default Todo app feel today,
- and does the remaining ceremony justify reopening shared API work?

This note records the answer from the current first-party default ladder rather than from an older
comparison target or an advanced/editor-grade surface.

## Audited evidence

Primary default-path surfaces:

- `docs/examples/todo-app-golden-path.md`
- `apps/fretboard/src/scaffold/templates.rs`
- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-examples/src/todo_demo.rs`

Closeout decisions reused here:

- action write budget is already closed on the current `cx.actions()` family
- query read posture is already closed on `cx.data().query(...)` + `handle.read_layout(cx)`
- query invalidation shell is already closed on grouped `cx.data().invalidate_query*`

## Current experience verdict

The current default Todo authoring experience is:

- coherent,
- productized,
- and clearly better than the earlier "capability exists but the first contact is too wide" state,
- but it is still not a shortest-possible toy-app surface in the way egui or JSX-first stacks can
  feel.

That is an acceptable result for the current ladder because the repo already teaches:

1. `hello` for the smallest runnable surface,
2. `simple-todo` for view runtime + typed actions + keyed lists,
3. `todo` for the selector/query baseline once derived and async state are actually needed.

In other words:

- `todo` should be judged as the third rung,
- not as the shortest possible first rung.

## Findings

### 1. App startup and shell composition are no longer the main problem

The first-contact startup story is now compact and productized:

- `FretApp::new(...).window(...).view::<TodoView>()?.run()`
- optional app installation stays on `.setup(...)`
- optional app-owned assets stay on the same builder chain

Within the view body, shell composition still uses explicit layout wrappers, but the audit result
from this lane remains correct:

- cookbook scaffolds already prove the default page-shell answer,
- local helper extraction such as `todo_page(...)` is sufficient,
- and the remaining wrapper noise does not justify widening the `fret` facade again.

Conclusion:

- composition density is still visible,
- but it is no longer the highest-value shared API target.

### 2. The state/action path is explicit, but no longer confused

The default Todo path now reads with one stable mutation model:

- local view-owned state via `cx.state().local*`
- coordinated writes via `cx.actions().locals::<A>(...)`
- simple one-slot writes via `local_set` / `local_update`
- keyed row writes via `payload_local_update_if::<A>(...)`

This is still more explicit than the shortest UI frameworks because Fret keeps:

- typed action IDs,
- explicit transaction boundaries,
- and explicit keyed payload routing.

But the remaining cost is mostly real mutation logic rather than helper taxonomy drift.

Conclusion:

- the action surface is not currently the reason Todo feels heavy,
- and reopening default write helpers from Todo pressure alone would be the wrong move.

### 3. Selector usage is now acceptable for a third-rung example, but it is still dense

The current selector posture is much better than the earlier raw dependency choreography:

- default LocalState-first path is `cx.data().selector_layout(...)`
- first-party docs no longer teach raw `DepsBuilder` as the default selector story

Even so, selector usage is still one of the more visibly dense parts of the `todo` template because
authors still need to make a few explicit choices:

- what the dependency tuple is,
- what the derived snapshot type is,
- and what should be pre-shaped before rendering keyed rows.

That cost is defensible on the third rung because selector introduction is the point of the
example. It would be much less acceptable if this were still the second rung.

Conclusion:

- selector ergonomics are not blocking the current ladder,
- but they remain one of the largest visible differences versus shorter authoring stacks.

### 4. Query usage is now grouped correctly; remaining verbosity is mostly lifecycle policy

The default query story is now coherent across create, read, and invalidate:

- create with `cx.data().query(...)`
- read with `handle.read_layout(cx)`
- invalidate with `cx.data().invalidate_query(...)` or
  `cx.data().invalidate_query_namespace(...)`

That means the old accidental shell is gone from the default app lane.

What remains in the `todo` template is mostly explicit lifecycle work:

- choosing a `QueryKey`
- choosing `QueryPolicy`
- deciding what loading/error/success should render
- deciding what local event should trigger refetch or cache invalidation

Conclusion:

- query authoring is no longer blocked by missing grouped app-lane helpers,
- and the remaining verbosity is mostly application policy rather than surface drift.

### 5. Router should still not be pulled into Todo ergonomics

Nothing in the current default Todo path depends on router ergonomics.

If router later wants a shorter app-shell posture, that should reopen as a router-owned question,
not as part of Todo or generic app composition density.

## What still feels heavier than other frameworks

Compared with shorter Rust UI authoring surfaces, the main remaining weight is now concentrated in
three places:

1. explicit typed action setup and handler registration,
2. explicit selector shaping for derived collections or counters,
3. explicit query lifecycle rendering rather than recipe-driven success/loading shortcuts.

This is a different class of issue from the already-closed helper-family ambiguity. It is not
primarily about the repo teaching the wrong API anymore.

## Decision from this audit

Do not reopen a broad default app-lane workstream from the current Todo evidence alone.

Instead:

- keep `hello` -> `simple-todo` -> `todo` as the stable ladder,
- keep the shipped `cx.actions()` / `cx.data()` grouped vocabulary stable,
- keep router out of this lane,
- and treat the current Todo density as mostly intentional explicitness plus third-rung teaching
  cost.

If future evidence justifies more work, it should reopen as a narrower question such as:

- recipe-level lifecycle presentation helpers in ecosystem crates,
- first-party scaffold/content discipline,
- or a targeted selector ergonomics study with non-Todo proof surfaces.

It should not reopen as another broad shared-facade helper expansion just because the Todo template
is not as short as a toy-app framework.
