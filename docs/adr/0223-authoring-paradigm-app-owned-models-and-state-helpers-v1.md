# ADR 0223: Authoring Paradigm (App-Owned Models) and State Helpers (v1)

Status: Accepted

## Context

Fret’s kernel is intentionally small and mechanism-only:

- app-owned mutable state via `Model<T>` (ADR 0031),
- explicit model/global observation and invalidation (ADR 0051),
- portable execution and driver-boundary inbox draining (ADR 0175),
- policy-heavy interactions live in ecosystem crates, not `crates/fret-ui` (ADR 0066 / ADR 0074).

However, the repository still exhibits “authoring drift” in demos and templates:

- stringly command routing (`"todo.remove.{id}"` parsing),
- ad-hoc async caches (each crate invents inboxes, dedupe, cancellation, GC),
- coarse “force refresh” hacks (e.g. `tick: Model<u64>` to invalidate caches).

These patterns make it harder for:

- new app authors to get started,
- ecosystem crates to share consistent conventions,
- third-party crates to integrate without learning “local folklore”.

We need to lock a **default authoring paradigm** and define a small set of **first-party ecosystem
helpers** that make the common cases easy, while keeping the kernel stable and portable.

## Decision

### D1 — Adopt an App-owned, handle-based authoring paradigm (GPUI-style)

Fret’s primary authoring model remains:

- **state is owned by the app** (the model store),
- callers hold typed **handles** (`Model<T>`) rather than owning the state,
- read/write access occurs only when a host (`App`) is available (`read`/`update` closures),
- state mutation remains **main-thread-only**.

This is an explicit choice to align with Rust’s ownership model and prevent “leaking borrows” across
frames or threads.

This ADR does **not** change the existing kernel contract (ADR 0031); it clarifies that this is
the canonical paradigm for user-facing authoring and ecosystem design.

### D2 — Keep the kernel minimal; put authoring ergonomics in the ecosystem

`crates/fret-ui` remains mechanism-only (ADR 0066). “Convenience” and policy surfaces live in
`ecosystem/*` so they can evolve without forcing kernel churn.

This ADR explicitly treats the following as **first-party ecosystem** surfaces (recommended defaults,
but replaceable by third-party alternatives):

- `ecosystem/fret-authoring` — shared authoring contracts (`UiWriter`) and optional state adapters for
  authoring frontends.
- `ecosystem/fret-executor` — portable background work + inbox helpers.
- `ecosystem/fret-query` — async resource state (TanStack Query-like) adapted to ADR 0175.
- `ecosystem/fret-selector` — memoized derived state helpers.
- `fret-kit::mvu::MessageRouter<M>` — typed routing for dynamic per-item actions.
- `ecosystem/fret-imui` — immediate-mode authoring facade that compiles to the declarative element tree
  (optional frontend; uses the same state helpers via `UiWriter` escape hatches and adapters).

### D3 — Standardize “state management” into three separate layers (do not merge)

We treat “state management” as three distinct capabilities with different contracts:

1. **Local mutable state (writeable):**
   - `Model<T>` (app/window state),
   - element-owned state via `ElementContext::with_state_*` (component state).
2. **Derived state (read-only, memoized):**
   - `Selector<Deps, TValue>` caches `TValue` behind an explicit dependency signature
     (`Deps: PartialEq`),
   - optional UI sugar (`use_selector`) stores selector caches in element state.
3. **Async resources (read-only state machine + cache):**
   - `QueryClient` manages `Model<QueryState<T>>` (UI observes it),
   - background fetch runs through `Dispatcher` + inbox drainers (ADR 0175),
   - completions apply only if the inflight token matches (stale results are ignored).

We explicitly do **not** attempt to build a single “Riverpod-like” umbrella framework. Each layer
must remain understandable and replaceable.

### D4 — Concurrency boundary: data-only background work, driver-boundary apply

All background work must respect the kernel’s execution model (ADR 0175):

- background tasks produce **data-only results**,
- results cross the boundary via an inbox drained at a driver boundary,
- application of results happens on the main thread and updates `Model<T>` values.

We do not assume:

- threads (`ExecBackgroundWork` may degrade to cooperative execution),
- or a specific async runtime (tokio/async-std).

If a platform cannot run real background threads (e.g. wasm), semantics should remain consistent,
even if performance degrades.

### D5 — Typed messages over stringly routing; stable `CommandId` only where needed

We standardize command routing conventions:

- **stable** `CommandId`s remain for menu/keybinding integration,
- **dynamic per-item actions** use typed routing (`MessageRouter<M>`) instead of parsing strings.

This is a direct response to recurring demo/template drift and is treated as part of the default
authoring story.

## Consequences

### Benefits

- **Rust-aligned safety:** no leaked borrows; updates are scoped to `read/update` closures.
- **Portable concurrency:** only depends on `Dispatcher.exec_capabilities()` (ADR 0175).
- **Ecosystem consistency:** shared derived/async patterns reduce duplicated “mini frameworks”.
- **Replaceability:** third-party crates can provide alternative selector/query systems as long as
  they integrate at the driver boundary and update observable state.

### Costs / Risks

- **More concepts than “signals-only” frameworks:** the golden path must be documented and taught.
- **Selector misuse risk:** hook-like sugar must provide guard rails (keys, deps helpers, diagnostics).
- **Async ecosystem friction:** async fetch works via optional spawners (tokio/wasm), but requires
  installing `FutureSpawnerHandle` and preserving driver-boundary apply semantics.

## Alternatives Considered

- **MVU-only (single `update(msg)` paradigm):**
  - Pros: very teachable; test-friendly.
  - Cons: derived state and async resources still need conventions; often devolves into boilerplate
    or ad-hoc caches at scale.
- **Signals/resources as the primary paradigm (reactive graph):**
  - Pros: great ergonomics for derived values and async resources.
  - Cons: introduces implicit dependency tracking and runtime complexity that conflicts with Fret’s
    “explicit invalidation + driver boundary” explainability goals.
- **Tokio-first / async-runtime-first design:**
  - Pros: integrates easily with Rust network/database ecosystems.
  - Cons: reduces portability; conflicts with the `Dispatcher` contract and wasm/threadless targets.

## Migration Plan

Tracking and adoption work is captured in:

- `docs/workstreams/state-management-v1.md` (state layers + typed routing + query adoption)
- `docs/workstreams/state-management-v1-todo.md` (concrete tasks and evidence anchors)
- `docs/workstreams/authoring-paradigm-gpui-style-v1.md` (authoring story consolidation)
- `docs/workstreams/authoring-paradigm-gpui-style-v1-todo.md` (milestones)

## Open Questions (Decision Gates)

### Query (async resource state)

- **Key conventions:** namespace + structured key patterns (what is stable, what is hashed, and how
  collisions are handled/debugged).
  - Status: Implemented + documented.
  - Evidence: `docs/query-key-conventions.md`
  - Evidence: `ecosystem/fret-query/src/lib.rs`
- **Default policy:** `stale_time`/`cache_time` defaults, dedupe and cancel semantics, and which
  operations are “forced refetch” vs “mark stale”.
- **Error model:** use a typed `QueryError` (kind + message) so retry/UX can be consistent without
  leaking backend error types into UI.
  - Status: Implemented (`QueryError` + `QueryRetryPolicy`).
  - Evidence: `ecosystem/fret-query/src/lib.rs`
- **Async adapter story:** should `fret-query` add an optional `Future` fetch mode via ecosystem
  adapters (tokio + wasm), while preserving the driver-boundary apply contract?
  - Status: Implemented (async query variants + `FutureSpawnerHandle`).
  - Evidence: `ecosystem/fret-executor/src/lib.rs`
  - Evidence: `docs/integrating-tokio-and-reqwest.md`
  - Evidence: `docs/integrating-sqlite-and-sqlx.md`
- **Instrumentation:** a minimal tracing vocabulary for query lifecycle (start/finish/cancel, cache
  hits, stale decisions) and a path to “devtools-like” introspection in `fretboard diag`.

### Selector (derived state)

- **Ergonomics + safety rails:** `DepsBuilder`, keyed variants, and helper fns for common deps
  (model revision tokens, global change tokens) to reduce “forgot to observe” footguns.
  - Status: Implemented.
  - Evidence: `ecosystem/fret-selector/src/ui.rs`
- **Shared derived state:** should we provide an opt-in “computed model” helper (`Model<U>` updated
  from `Model<T>` changes) in addition to local memo selectors?
- **Debuggability:** tracing hooks for selector recompute + a way to surface unexpected recompute
  rates or missing deps in diagnostics bundles.

### Typed routing (commands/messages)

- **Prefix conventions:** per-window uniqueness and hotpatch reset behavior for `MessageRouter`.
- **Stable vs dynamic commands:** codify when to allocate stable `CommandId`s vs using typed routing.

### Concurrency boundary (ADR 0175)

- **Cancellation semantics:** standardize token usage and cancellation expectations across ecosystem
  crates (query, asset caches, indexing, etc.).
- **Capability degradation:** define which semantics must remain consistent when background work is
  not truly parallel (wasm/threadless).
