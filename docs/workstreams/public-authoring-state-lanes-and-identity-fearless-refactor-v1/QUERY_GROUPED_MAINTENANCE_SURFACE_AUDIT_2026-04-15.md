# Query Grouped Maintenance Surface Audit — 2026-04-15

Status: landed follow-on audit
Last updated: 2026-04-15

Related:

- `TODO.md`
- `MILESTONES.md`
- `docs/crate-usage-guide.md`
- `docs/authoring-golden-path-v2.md`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/tests/uicx_data_surface.rs`
- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/src/markdown_demo.rs`
- `docs/workstreams/app-composition-density-follow-on-v1/QUERY_INVALIDATION_SHELL_AUDIT_2026-03-17.md`

## Why this note exists

After the `AppUi` root accessor and `fret-ui-assets` adapter slices landed, the next candidate seam
was the remaining raw `with_query_client(...)` usage inside first-party app-facing examples.

The open question was not whether `fret-query` needed more engine APIs.
The real question was:

- should query snapshot/cancellation stay as raw app-local client shell code,
- or do they belong with the grouped app-facing query maintenance story that already owns
  invalidation on `cx.data()`?

## Audited evidence

Primary proof surfaces:

- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/src/markdown_demo.rs`

Owner and teaching posture evidence:

- `ecosystem/fret/src/view.rs`
- `docs/crate-usage-guide.md`
- `docs/authoring-golden-path-v2.md`
- `docs/workstreams/app-composition-density-follow-on-v1/QUERY_INVALIDATION_SHELL_AUDIT_2026-03-17.md`

Boundary evidence:

- `ecosystem/fret-authoring/src/query.rs`
- `ecosystem/fret-query/src/lib.rs`

## Finding 1: ownership is still the grouped app-facing `fret` lane, not `fret-query`

The earlier invalidation closeout already froze the owner rule:

- grouped app query maintenance belongs on `cx.data()` inside `fret`,
- raw `fret::query::with_query_client(...)` remains the pure app/driver seam,
- `fret-query` stays the portable engine crate.

This follow-on evidence does not justify reopening that split.

`async_playground_demo` was still using raw client shell code from `AppUi` / `UiCx`, but that was
authoring-surface drift, not missing engine ownership.

## Finding 2: query snapshot/cancel are the same maintenance family as invalidation on app-facing UI surfaces

`async_playground_demo` exercised three explicit query maintenance needs from grouped UI code:

- invalidate one key,
- cancel one inflight query,
- inspect snapshot metadata (`stale`, status, inflight) for status chrome.

Those operations all share the same characteristics on the app lane:

- they run inside `AppUi` or extracted `UiCx`,
- they already have a grouped `cx.data()` carrier,
- and they should not force app-facing code back to raw query-client plumbing just to reach the
  engine.

Conclusion:

- grouped app-facing query maintenance should cover snapshot reads and single-key cancellation too,
- not only invalidation.

## Finding 3: the existing public query noun set was already sufficient

`fret::query` already reexports:

- `QueryClientSnapshot`
- `QuerySnapshotEntry`
- `QueryKey`

So the missing piece was not noun visibility.
The missing piece was grouped access on the app-facing carrier.

That allows a narrow follow-on without widening `fret::app::prelude::*` or moving APIs into the
wrong crate.

## Landed result

This audit lands:

- grouped query maintenance/snapshot helpers on `fret::view`:
  - `cx.data().query_snapshot()`
  - `cx.data().query_snapshot_entry(...)`
  - `cx.data().cancel_query(...)`
- migration of the first-party proof surfaces:
  - `async_playground_demo` now uses grouped invalidation/cancel/snapshot helpers,
  - `markdown_demo` now uses grouped namespace invalidation too
- source-policy and docs updates so the repo now teaches the grouped lane consistently

## What remains explicit after this audit

This does **not** collapse all query-client usage into `cx.data()`.

Raw `fret::query::with_query_client(...)` still remains the correct explicit seam for:

- pure `&mut App` / driver code,
- lower-level generic writer extensions outside the `fret` app-facing facade,
- engine tests and engine-local lifecycle plumbing.

`ecosystem/fret-authoring/src/query.rs` remains valid evidence of that lower-level authoring layer.

## Decision from this audit

Treat app-facing query maintenance as one grouped family on `cx.data()`:

- invalidation,
- single-key cancellation,
- and snapshot inspection for app-facing diagnostics/status chrome.

Do **not** move that family into `fret-query`, and do **not** reopen the owner split unless fresh
evidence shows that grouped app-facing UI code still cannot express a real product surface after
this slice.
