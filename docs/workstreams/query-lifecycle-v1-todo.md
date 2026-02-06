# Query Lifecycle v1 (Tracking)

Last updated: 2026-02-06

This file tracks concrete work for the query lifecycle semantics described in:

- `docs/adr/1164-query-lifecycle-and-cache-semantics-v1.md`
- `docs/workstreams/query-lifecycle-v1.md`

Status legend:

- `[ ]` not started
- `[~]` in progress
- `[x]` done

## Phase 0 — ADR + documentation anchors

- `[x]` Draft ADR 1164 (stale/refetch/cancel/retry semantics).
  - Evidence: `docs/adr/1164-query-lifecycle-and-cache-semantics-v1.md`
- `[x]` Create the workstream doc + tracker.
  - Evidence: `docs/workstreams/query-lifecycle-v1.md`
  - Evidence: `docs/workstreams/query-lifecycle-v1-todo.md`

## Phase 1 — Core semantics (no implicit polling)

- `[x]` Change `fret-query` to ensure `stale_time` does not act as a polling interval under
  rebuild-every-frame authoring.
  - Add frame-based “remount” detection.
  - Add regression tests.
  - Target: `ecosystem/fret-query/src/lib.rs`
- `[ ]` Re-evaluate `QueryPolicy` defaults (cache time, stale time) after the semantic change.
  - Target: `ecosystem/fret-query/src/lib.rs`

## Phase 2 — Demo/template alignment

- `[ ]` Audit and update places that set `stale_time` expecting freshness semantics (not polling).
  - Candidates:
    - `apps/fret-examples/src/markdown_demo.rs` (remote image cache)
    - `ecosystem/fret-markdown/src/mathjax_svg_support.rs` (local deterministic cache)
    - `apps/fret-examples/src/query_demo.rs` (show invalidate/refetch)
- `[ ]` Add a short note in the query demo explaining the “remount + stale” trigger.

## Phase 3 — Integration guidance (reqwest/sqlx/wasm)

- `[x]` Update `docs/integrating-tokio-and-reqwest.md`:
  - clarify lifecycle semantics (stale != auto refresh),
  - show a common “mutation -> invalidate namespace -> refetch” flow,
  - show how to implement explicit polling when desired.
- `[ ]` Update `docs/integrating-sqlite-and-sqlx.md` with a `fret-query` invalidation pattern.

## Phase 4 — Diagnostics (devtools-lite)

- `[ ]` Add tracing events for:
  - fetch start/finish (hit/miss),
  - cancellation mode decisions,
  - retry schedule decisions,
  - GC evictions.
- `[ ]` Add a snapshot export API (data-only) for diagnostics bundles.
  - Candidate: `QueryClient::snapshot()` returning keys + policy + status metadata.
