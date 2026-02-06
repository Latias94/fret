# Query Lifecycle v1 (Workstream)

Status: Active (ecosystem-level; ADRs remain the source of truth)

Related ADR:

- `docs/adr/1164-query-lifecycle-and-cache-semantics-v1.md`

This workstream turns the “async resource state” story into a predictable, debuggable, portable
default for Fret apps by:

- locking down `fret-query` lifecycle semantics (no implicit polling),
- aligning demos/templates/docs with the new contract,
- documenting common Rust ecosystem integrations (`reqwest`, `sqlx`, wasm futures),
- adding minimal diagnostics hooks for query lifecycle visibility.

## Goals

1. **No implicit polling:** `stale_time` is freshness only; refetch is triggered only by explicit
   invalidation/refetch, retry policy, or remount+stale.
2. **Portable concurrency:** do not assume threads or a specific async runtime (ADR 0190).
3. **Predictable UI states:** Loading/Error/Success should be straightforward to render from
   `Model<QueryState<T>>`.
4. **Debuggability:** provide a minimal tracing vocabulary and a way to snapshot cache state.

## Non-goals (v1)

- A “Query Devtools” UI. We start with diagnostics bundle data + tracing.
- Automatic refetch on window focus / reconnect. These are useful, but require platform surfaces
  that should be designed intentionally.

## “Author experience” target

An app author should be able to:

- define a stable `QueryKey<T>` for each resource,
- pick a small `QueryPolicy` (cache time, stale time, retry, cancel semantics),
- render from `QueryState<T>` (loading/error/data),
- invalidate/refetch explicitly after mutations,
- integrate async fetch with Tokio or wasm by installing a `FutureSpawnerHandle` global.

Tracking lives in:

- `docs/workstreams/query-lifecycle-v1-todo.md`

