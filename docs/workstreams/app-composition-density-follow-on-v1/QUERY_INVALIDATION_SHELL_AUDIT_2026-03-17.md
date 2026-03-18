# Query Invalidation Shell Audit — 2026-03-17

Status: M2 audit note
Last updated: 2026-03-17

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `TODO.md`
- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`
- `apps/fret-examples/src/async_playground_demo.rs`
- `docs/integrating-tokio-and-reqwest.md`
- `docs/integrating-sqlite-and-sqlx.md`
- `ecosystem/fret-authoring/src/query.rs`

## Why this note exists

M2 asked whether the remaining default app-lane query invalidation shell was:

- an acceptable explicit advanced seam,
- only a docs problem,
- or a repeated grouped-surface gap on `AppUi` / extracted `UiCx`.

This note records the audit result and the landed direction.

## Audited evidence

Primary app-facing proof surfaces:

- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`

Secondary proof / boundary surfaces:

- `apps/fret-examples/src/async_playground_demo.rs`
- `docs/integrating-tokio-and-reqwest.md`
- `docs/integrating-sqlite-and-sqlx.md`
- `ecosystem/fret-authoring/src/query.rs`

## Finding 1: the shell was real and repeated on the default app lane

Before this batch, first-party app-facing query examples repeatedly spelled:

- `with_query_client(cx.app, |client, app| client.invalidate(app, key))`
- `with_query_client(cx.app, |client, _app| client.invalidate_namespace(ns))`
- `cx.app.request_redraw(cx.window)`

This was not a Todo-only pattern.

It appeared on:

- cookbook app-facing query basics,
- sync query demo,
- async query demo,
- and app-facing integration docs.

Conclusion:

- this was a real repeated app-lane shell, not one example's local noise.

## Finding 2: the lower-level authoring stack already proved the helper shape

`ecosystem/fret-authoring/src/query.rs` already shipped:

- `invalidate_query(...)`
- `invalidate_query_namespace(...)`

on a generic authoring writer surface.

That was important evidence because it showed:

- grouped invalidation is compatible with Fret's query ownership model,
- the redraw shell belongs near the authoring surface,
- and `fret-query` itself does not need to absorb app-facing sugar to support this.

Conclusion:

- the likely missing piece was productizing the same shape on the default app-facing `fret`
  facade, not inventing a brand-new concept.

## Finding 3: ownership belongs on `cx.data()`, not `fret-query`

The landed grouped helper now lives on:

- `AppUiData`
- `UiCxData`

with these spellings:

- `cx.data().invalidate_query(key)`
- `cx.data().invalidate_query_namespace(namespace)`

The helper owns the redraw shell for the app-facing UI context, while pure `&mut App` /
driver-boundary code still uses raw `fret::query::with_query_client(...)`.

Conclusion:

- this keeps grouped app UI code coherent,
- preserves `fret-query` as the portable engine crate,
- and avoids widening the default query story into a new parallel API family.

## Finding 4: router remains out of scope

Nothing in this audit changed the router posture.

`apps/fret-cookbook/examples/router_basics.rs` remains an explicit router-owned surface, and this
batch did not introduce any generic invalidation glue there.

Conclusion:

- router remains outside this lane.

## Landed result

This audit batch lands:

- grouped app-facing invalidation helpers on `cx.data()`,
- migration of first-party app-facing query examples to those helpers,
- docs updates that distinguish app-lane grouped invalidation from pure app/driver raw invalidation,
- source-policy tests guarding the new grouped story.

## What remains after this audit

This note does **not** close the entire workstream.

Remaining open work is still:

- M1 app-shell composition density,
- and M3 delete/lock cleanup after any further composition decision.

But M2's query invalidation question is now resolved for the default app lane.

## Decision from this audit

Treat M2 as closed for the default app-lane query invalidation shell:

- grouped invalidation belongs on `cx.data()` for `AppUi` / extracted `UiCx`,
- raw `with_query_client(...)` remains the explicit pure app/driver seam,
- and no broader query/write surface redesign is justified by this question.
