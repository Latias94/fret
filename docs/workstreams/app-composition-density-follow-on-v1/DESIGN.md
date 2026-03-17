# App Composition Density Follow-on v1

Status: active planning lane
Last updated: 2026-03-17

Related:

- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/QUERY_READ_SURFACE_CLOSEOUT_2026-03-17.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/ECOSYSTEM_ADAPTATION_AND_ROUTER_AUDIT_2026-03-17.md`
- `docs/workstreams/action-write-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-17.md`
- `docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- `docs/examples/todo-app-golden-path.md`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`
- `ecosystem/fret-authoring/src/query.rs`

## Why this workstream exists

The broad pre-release authoring reset is already closed on its main questions:

- the app/component/advanced lane split is settled,
- the default app-lane read surface is settled on `cx.data().selector_layout(...)`,
  `cx.data().query(...)`, and `handle.read_layout(cx)`,
- the default app-lane write budget is settled on the current `cx.actions()` families,
- and router remains an explicit adjacent seam rather than a driver for default-path sugar.

The remaining pain is narrower.

Today, the third-rung Todo path is structurally correct and teachable, but the default app lane
still shows two kinds of accidental ceremony:

1. page/root composition often over-spells layout transport with nested wrapper closures,
2. query invalidation still drops back to raw `with_query_client(...)` + `request_redraw(...)`
   shell code even on first-party app-facing examples.

This workstream exists to reduce that accidental ceremony without reopening:

- selector/query read-surface design,
- the `cx.actions()` write budget,
- the app/component/advanced taxonomy,
- or router ergonomics.

## Current diagnosis

### 1. App-shell composition is correct but still denser than it needs to be

The repo already converged on the right primitives:

- `ui::single(cx, child)` for single-child landing,
- `ui::children![cx; ...]` for child collections,
- `ui::for_each_keyed(...)` for keyed lists,
- typed `impl UiChild` helpers for page shells and wrappers.

Even after that cleanup, the default app lane still repeats wrapper-only composition like:

- `ui::container(move |cx| { ui::single(cx, ...) })`
- `ui::v_flex(move |cx| ui::single(cx, content))`
- card/body shells that mostly transport one already-composed child through another closure layer

Those seams are not wrong, but they still make ordinary app code feel more ceremonial than the
already-settled runtime/dataflow model requires.

### 2. Query invalidation still falls back to raw client plumbing on the app lane

The app-facing read path is already grouped and productized:

- `cx.data().query(...)`
- `handle.read_layout(cx)`

But the corresponding app-lane write-side query maintenance path is not grouped the same way.
Current first-party examples still teach:

- `with_query_client(cx.app, |client, app| client.invalidate(app, key))`
- `with_query_client(cx.app, |client, _app| client.invalidate_namespace(namespace))`
- `cx.app.request_redraw(cx.window)`

That repeated shell appears in:

- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`
- `docs/integrating-sqlite-and-sqlx.md`
- `docs/integrating-tokio-and-reqwest.md`

There is already a lower-level authoring precedent for narrowing this shell in
`ecosystem/fret-authoring/src/query.rs`, which provides `invalidate_query(...)` and
`invalidate_query_namespace(...)` on a generic authoring writer surface.

That is useful evidence:

- the shell is real and repeated,
- it is not Todo-only pressure,
- and the likely ownership layer is the app-facing facade or authoring layer, not `fret-query`
  itself.

### 3. Router should stay out of this lane

`apps/fret-cookbook/examples/router_basics.rs` still reads as intentionally explicit:

- `RouterUiStore`
- typed `RouteCodec`
- explicit `NavigationAction`
- `RouterOutlet`

That ceremony is capability ownership, not accidental app-lane density.

If router later needs a shorter app-shell story, it should reopen as a router-specific lane rather
than widening this workstream.

## Fearless-refactor rules

This remains a pre-release lane.

That means:

- delete displaced first-contact spellings once a better one lands,
- do not preserve compatibility aliases just because they existed,
- and do not reopen settled surface families only to shave one demo.

## Scope

### In scope

- default app-lane page/root/shell composition density
- wrapper-only single-child transport patterns on first-party app-facing examples
- default app-lane query invalidation shell
- docs/templates/examples/source-gate alignment for whichever narrower posture lands

### Explicit non-goals

- redesigning selector/query read surfaces
- redesigning `cx.actions()` or payload-write helpers
- reopening app/component/advanced taxonomy
- widening `fret::app::prelude::*`
- router ergonomics or route authoring sugar
- component/advanced/interop helper cleanup outside the default app lane
- shadcn family recipe redesign
- DSL / macro / JSX-style authoring
- `LocalState<T>` storage-model redesign

## Evidence set

### Primary detection surface

- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-cookbook/examples/simple_todo.rs`
- `docs/examples/todo-app-golden-path.md`
- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`

### Secondary proof / migration surface

- `docs/integrating-sqlite-and-sqlx.md`
- `docs/integrating-tokio-and-reqwest.md`
- `apps/fret-examples/src/async_playground_demo.rs`
- `ecosystem/fret-authoring/src/query.rs`

### Excluded as design drivers

- `apps/fret-cookbook/examples/router_basics.rs`
- router workstream docs under `docs/workstreams/router-v1/` and `docs/workstreams/router-ui-v1/`

These remain useful boundary checks, but they should not drive default app-lane helper design here.

## Design constraints

1. Keep the grouped app nouns stable.
   This lane may shorten composition and query invalidation, but it must still read as
   `cx.state()`, `cx.actions()`, `cx.data()`, and `cx.effects()`.
2. Keep `ui::single(cx, child)` as the default single-child landing rule unless evidence shows a
   narrower wrapper helper is genuinely better.
3. Prefer deleting wrapper-only ceremony over inventing a second composition dialect.
   If local helper extraction or stricter first-party wrapper rules solve the problem, prefer that
   over broad new API.
4. Keep query lifecycle nouns explicit.
   A narrower invalidation shell must still keep key-vs-namespace choice and async ownership
   visible.
5. Keep ownership at the app-facing facade layer.
   If a grouped query invalidation helper lands, it belongs in `ecosystem/fret` or another
   app-facing authoring layer, not in `fret-query`.
6. Do not use this lane to normalize router, advanced, or component surfaces toward the app lane.

## What success means

This workstream is successful when:

- Todo-sized default app code is visibly shorter for the right reasons,
- at least one non-Todo app-facing surface benefits from the same reduction,
- selector/query read surface and action surface stay unchanged,
- router remains explicit and unaffected,
- and the repo can hard-delete the displaced first-contact wording instead of teaching two
  co-equal stories.

## Execution order

1. Freeze scope, evidence, and exclusions.
2. Audit repeated app-shell composition patterns and decide docs-only vs narrow helper.
3. Audit repeated query invalidation shell and decide whether a grouped app-lane helper is
   justified.
4. Update docs/examples/templates/gates together, then delete displaced wording.
