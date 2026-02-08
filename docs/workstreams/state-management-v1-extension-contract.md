# State Management v1: Extension Boundaries and Ecosystem Integration

Status: Draft (implementation guidance; ADRs remain the source of truth)

Related:

- `docs/workstreams/state-management-v1.md`
- `docs/adr/1162-authoring-paradigm-app-owned-models-and-state-helpers-v1.md`
- `docs/adr/1164-query-lifecycle-and-cache-semantics-v1.md`
- `docs/integrating-tokio-and-reqwest.md`

This document defines the **official extension boundaries** for `fret-selector` and `fret-query`,
plus a practical integration contract for common ecosystem stacks (`reqwest`, `sqlx`, GraphQL,
websocket streams).

## 1. Scope and non-goals

Scope:

- clarify what should be considered stable extension points in v1;
- keep async/concurrency semantics portable across native + wasm;
- provide a default, copyable integration pattern for app teams and third-party crates.

Non-goals:

- introducing a global DI/container framework;
- forcing one async runtime across all hosts;
- replacing app/domain state modeling (`Model<T>` remains app-owned).

## 2. Official extension boundaries (v1)

### 2.1 Local mutable state (baseline)

- Official primitive: `Model<T>` + explicit invalidation in `ElementContext` reads.
- Contract: all `ModelStore` mutations happen on the main thread.
- Extension rule: third-party crates may own their own models, but must expose data-only APIs back
  to app code.

### 2.2 Derived state (`fret-selector`)

- Official primitive: `Selector<Deps, TValue>` and `use_selector(...)` UI sugar.
- Stable boundary:
  - `Deps` is explicit and comparable (`PartialEq`),
  - selector compute closure returns pure derived value (no side-effect),
  - recomputation is driven by dependency change only.
- Extension rule:
  - selectors may read models/globals but must not hold long-lived store borrows,
  - do not hide I/O inside selector closures.

### 2.3 Async resource state (`fret-query`)

- Official primitive: `QueryClient`, `QueryKey`, `QueryPolicy`, `QueryState<T>`.
- Stable boundary:
  - fetch runs off-thread or off-main-loop through a spawner handle,
  - completion is applied on the main thread via inbox-drain boundaries,
  - stale inflight completions are ignored by inflight token checks.
- Extension rule:
  - use `use_query_async(...)` for `Send + 'static` futures,
  - use `use_query_async_local(...)` for wasm/`!Send` futures,
  - keep query keys deterministic and namespace-scoped.

### 2.4 Typed command routing (UI intent boundary)

- Official primitive:
  - `MessageRouter<M>` for per-frame dynamic commands,
  - `KeyedMessageRouter<K, M>` for view-cache-safe dynamic commands.
- Contract:
  - use typed messages for app mutations,
  - reserve literal `CommandId` strings for globally addressable keymap/menu actions.

## 3. Memory-safety and concurrency model (must keep)

Hard constraints for any extension crate:

- do not mutate `App`/`ModelStore` from background tasks;
- transport only data across threads/tasks (inbox messages), then apply on UI thread;
- treat cancellation as best-effort and idempotent;
- make background producers tolerant to dropped/ignored results.

Practical API guidance:

- separate “fetch/compute” from “apply to models” into two phases;
- require `Clone + Send + 'static` only where truly needed;
- provide both native (`Send`) and wasm-local (`!Send`) entry points when possible.

## 4. Third-party ecosystem integration recipes

### 4.1 HTTP APIs (`reqwest`)

Recommended stack:

- request/deserialize in `use_query_async(...)` fetch closure,
- map transport errors to `QueryError::{transient, permanent}`,
- invalidate namespace after successful mutation.

When to use query vs manual inbox:

- query: snapshot-like resources (lists/details/config);
- manual inbox/stream pipeline: high-frequency pushes (progress/log/event stream).

### 4.2 SQL (`sqlx`/SQLite)

Recommended split:

- query for read models (`list_tasks`, `load_settings`),
- command/inbox pipeline for writes and transactional workflows,
- post-write invalidate related query namespaces.

### 4.3 GraphQL clients

Recommended split:

- query keys include operation name + normalized variables;
- short `stale_time` for highly-shared collaborative views;
- mutation completion triggers namespace invalidation.

### 4.4 SSE/WebSocket/Realtime

Treat as a stream source, not as polling query:

- background task receives events and sends data-only messages,
- app reduces messages into `Model<T>`,
- optionally invalidate queries that mirror eventually-consistent snapshots.

## 5. Official vs third-party crate guidance

### 5.1 Official ecosystem crates (`ecosystem/*`)

Default expectation:

- prefer `fret-selector` for non-trivial derived state reuse,
- prefer `fret-query` for async resource lifecycle,
- expose typed APIs and avoid string command parsing in public examples.

### 5.2 Third-party ecosystem crates

Recommended publishing strategy:

- make `fret-selector` / `fret-query` optional features (`state-selector`, `state-query`),
- keep a small core API independent from state stack choice,
- expose adapter traits or callbacks so host apps can integrate their own state layer.

### 5.3 Immediate-mode wrappers (`imui`-style)

Recommended integration mode:

- use `QueryClient` as service-first API outside `ElementContext` sugar,
- run selector/query computations in host app state and pass plain values into immediate draws,
- keep command/event mapping typed at wrapper boundary.

## 6. Adoption checklist

For new app templates and AI-assisted generation:

1. Use typed message routing (`MessageRouter` or `KeyedMessageRouter` by cache mode).
2. Use `Selector` for derived counters/filters/projections.
3. Use `QueryClient` for async resource loading/error/retry/cache.
4. Keep writes in command handlers and trigger query invalidation explicitly.
5. Install runtime spawner once (tokio/wasm) and keep fetch closures pure.

## 7. Open decisions to lock before v1 freeze

- query key namespace conventions across official crates (cross-crate collision policy);
- default retry presets by domain (network, disk, parsing);
- whether to ship a minimal stream-state helper next to `fret-query` or keep it app-owned;
- how far template scaffolding should go (single “todo baseline” vs multiple variants).
