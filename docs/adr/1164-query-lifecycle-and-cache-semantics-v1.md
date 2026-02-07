# ADR 1164: Query Lifecycle and Cache Semantics (v1)

Status: Proposed

## Context

Fret’s kernel is intentionally small and mechanism-only:

- app-owned mutable state via `Model<T>` (ADR 0031),
- explicit model/global observation and invalidation (ADR 0051),
- portable execution and driver-boundary inbox draining (ADR 0190),
- policy-heavy authoring ergonomics live in ecosystem crates, not `crates/fret-ui` (ADR 0066).

`ecosystem/fret-query` provides “async resource state” similar to TanStack Query, but adapted to
these constraints:

- query state lives in `Model<QueryState<T>>` so UI can observe it,
- background work produces data-only results and crosses a driver boundary,
- apply happens on the UI thread and stale results are ignored via inflight tokens.

However, without an explicit lifecycle contract, it is easy for apps and demos to drift into
undesirable behavior. The most common footgun is treating `stale_time` as an implicit polling
interval when `use_query(...)` is called every frame (declarative element tree rebuild), causing
unexpected background work and network I/O.

We need to lock down a v1 lifecycle contract that:

- is predictable and debuggable,
- is portable (does not assume threads or a specific async runtime),
- avoids implicit polling and hidden retries,
- stays ecosystem-level and replaceable.

## Decision

### D1 — `stale_time` defines freshness, not implicit polling

`QueryPolicy.stale_time` defines when data becomes **stale** (no longer “fresh”). Becoming stale
must **not** automatically refetch by itself.

If polling is desired, it must be explicit (e.g. a timer effect that calls `refetch(...)` or
`invalidate(...)`).

### D2 — Default refetch triggers (v1)

A query fetch may start only when one of the following triggers occurs:

1. **Initial use**: the query is `Idle` (no successful result yet).
2. **Explicit invalidation**: `QueryClient::invalidate(...)` or `invalidate_namespace(...)` marks
   the key stale.
3. **Explicit refetch**: `QueryClient::{refetch,refetch_async,refetch_async_local}(...)` forces a
   new request even if the data is fresh.
4. **Retry**: retry policy schedules a follow-up attempt after a transient error.
5. **Remount + stale**: if the query becomes active again after being unobserved for at least one
   frame, and the cached result is stale by `stale_time`, a new fetch may start.

This preserves a TanStack-like mental model (stale gates refetch on “observer attach”), without
turning “called every frame” into polling.

### D3 — “Observer activity” is frame-based and explicit

`fret-query` must treat a query as “continuously observed” when `use_query*` is called in
consecutive frames for that key.

When `use_query*` is called after a gap of **more than one frame** (based on `UiHost::frame_id()`),
the query is considered **remounted** for the purposes of D2.5.

This definition is portable and does not depend on a specific UI frontend:

- declarative element authoring calls `use_query*` during view build,
- immediate-mode frontends rebuild every frame and therefore keep queries observed,
- view-cache reuse may skip the builder closure; see “Caveats”.

### D4 — Concurrency and cancellation semantics are stable

`QueryPolicy` defines concurrency behavior:

- `dedupe_inflight=true` means only one inflight request is allowed per key; additional uses do not
  start a second request.
- `cancel_mode=CancelInFlight` means a superseding request cancels the previous task (best-effort);
  `KeepInFlight` allows multiple inflight tasks but only the latest completion is applied.
- Stale completions must be ignored using inflight IDs.

### D5 — Retry is explicit and scoped to transient failures by default

Retry policy is configured per query via `QueryPolicy.retry`:

- `QueryErrorKind::{Transient,Permanent}` is used to classify errors,
- retry defaults to `None`,
- when enabled, retry defaults to transient errors only.

### D6 — Async integration is ecosystem-level via `FutureSpawnerHandle`

`fret-query` may support async fetch functions, but must not require a specific runtime.

- Apps install a `FutureSpawnerHandle` global (Tokio / wasm adapters live in ecosystem crates).
- `use_query_async` requires `Send` futures.
- `use_query_async_local` supports `!Send` futures (typically wasm).

### D7 — Scope and replaceability

This ADR defines the semantics for the **first-party ecosystem** query helper. It does not require
kernel changes. Third-party alternatives are free to exist as long as they:

- update observable state (`Model<T>`),
- respect the driver-boundary apply contract (ADR 0190),
- keep query keys stable and debuggable.

## Caveats

### View cache reuse

`view_cache(...)` roots may reuse a subtree and skip running the subtree builder closure. If a
query’s `use_query*` call lives only inside such a subtree, then:

- invalidation may not immediately schedule a refetch (because no observer call runs),
- “remount” detection may be delayed until the subtree is rebuilt.

Recommended patterns:

- call `use_query*` in an uncached parent and pass the `QueryHandle` down, or
- drive invalidation/refetch from the app/update path (explicit commands/effects), not only from
  view code.

### `stale_time` is not “auto refresh”

If an app needs periodic refresh, it should model it explicitly:

- store a `TimerToken` or `Effect::RequestAnimationFrame` lease,
- on tick, call `QueryClient::refetch(...)` / `invalidate(...)`,
- keep the behavior visible and debuggable.

## Common integration scenarios (non-normative)

- **HTTP (desktop):** Tokio + reqwest + `use_query_async` (see `docs/integrating-tokio-and-reqwest.md`).
- **HTTP (no async runtime):** blocking fetch in a background task via `use_query` + `ureq`.
- **Database:** `sqlx` queries via `use_query_async` (Tokio) with explicit invalidation after
  mutations (see `docs/integrating-sqlite-and-sqlx.md`).
- **Assets/caches:** remote images, syntax bundles, plugin registries: stable keys + long cache time
  + explicit invalidation when inputs change.

## Alternatives considered

- **Treat `stale_time` as a polling interval:** simple, but causes hidden background work and
  surprises in declarative rebuild-every-frame authoring.
- **Signals/resources as a global reactive graph:** ergonomic, but conflicts with explicit
  invalidation + driver-boundary explainability goals.
- **Tokio-first design:** integrates easily with Rust async ecosystems, but reduces portability and
  conflicts with ADR 0190’s capability degradation model.

## Migration plan

- Update `ecosystem/fret-query` to implement D1–D6 semantics.
- Add tests to prevent “stale_time implies polling” regressions.
- Update demos and docs that currently set `stale_time` intending “freshness” semantics.
- Track follow-ups in `docs/workstreams/query-lifecycle-v1-todo.md`.

