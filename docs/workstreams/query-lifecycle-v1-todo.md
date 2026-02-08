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
- `[x]` Re-evaluate `QueryPolicy` defaults (cache time, stale time) after the semantic change.
  - Decision: keep `stale_time = 2s` and `cache_time = 60s` as defaults.
  - Reasoning: after ADR 1164 semantics, `stale_time` only gates freshness and no longer implies implicit polling.
  - Guidance: for realtime/high-frequency views, use explicit polling or invalidate/refetch flows.
  - Evidence: `ecosystem/fret-query/src/lib.rs` and remount boundary tests.

## Phase 2 — Demo/template alignment

- `[x]` Audit and update places that set `stale_time` expecting freshness semantics (not polling).
  - `apps/fret-examples/src/markdown_demo.rs`: add explicit `Refresh remote images` action via `invalidate_namespace`; align policy to freshness window semantics.
  - `ecosystem/fret-markdown/src/mathjax_svg_support.rs`: align deterministic local cache policy (`stale_time == cache_time`) to make lifecycle intent explicit.
  - `apps/fret-examples/src/query_demo.rs`: migrate to typed messages and keep invalidate/refetch guidance in UI copy.
- `[x]` Add a short note in the query demo explaining the “remount + stale” trigger.
  - Evidence: `apps/fret-examples/src/query_demo.rs`

## Phase 3 — Integration guidance (reqwest/sqlx/wasm)

- `[x]` Update `docs/integrating-tokio-and-reqwest.md`:
  - clarify lifecycle semantics (stale != auto refresh),
  - show a common “mutation -> invalidate namespace -> refetch” flow,
  - show how to implement explicit polling when desired.
- `[x]` Update `docs/integrating-sqlite-and-sqlx.md` with a `fret-query` invalidation pattern.
  - Evidence: `docs/integrating-sqlite-and-sqlx.md`

## Phase 4 — Diagnostics (devtools-lite)

- `[x]` Add tracing events for:
  - fetch start/finish,
  - cancellation mode decisions,
  - retry schedule decisions,
  - GC evictions.
  - Evidence: `ecosystem/fret-query/src/lib.rs` (`target = "fret_query::diag"`)
- `[x]` Add a snapshot export API (data-only) for diagnostics bundles.
  - Implemented: `QueryClient::snapshot()` with deterministic entry ordering.
  - Evidence: `ecosystem/fret-query/src/lib.rs`
  - Bundle integration: `UiDiagnosticsSnapshotV1.query_snapshot` includes query snapshot metadata for `bundle.json` exports.
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- `[x]` Surface query snapshot aggregates in `fretboard diag stats` reports.
  - Added human/json metrics: `frames_with_query_snapshot`, `query_entries_*`, `query_namespace_hotspots`.
  - Evidence: `crates/fret-diag/src/stats.rs`
  - Regression coverage: `bundle_stats_aggregates_query_snapshot_entries` in `crates/fret-diag/src/lib.rs`.
