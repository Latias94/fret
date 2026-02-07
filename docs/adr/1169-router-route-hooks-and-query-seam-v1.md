# ADR 1169: Router Route Hooks and `fret-query` Seam (v1)

Status: Proposed

## Context

`ecosystem/fret-router` is intentionally lightweight, but it still needs a stable contract for the
“hard to change” parts of routing that app and ecosystem layers will build on:

- nested matching (root -> leaf match chain)
- search validation and stabilization
- transitions and guard outcomes (ADR 1168)
- route-scoped lifecycle hooks (TanStack-style `beforeLoad`/`loader`)
- a predictable seam for data loading/prefetching via `ecosystem/fret-query`

TanStack Router’s core concepts we want to align with (outcomes, not implementation):

- `validateSearch`: route-level search validation/stabilization
- `beforeLoad`: route-level pre-navigation policy/middleware that can block/redirect
- `loader`: route-level data preloading/prefetching to avoid waterfalls

In Fret, the router must remain portable (native + wasm) and must not impose an async runtime on
apps. `fret-query` is the preferred ecosystem data loading backend, but query keys are typed
(`QueryKey<T>`), so router-core cannot fully “plan keys” without application knowledge of `T`.

## Decision

### 1) Split policy surfaces by responsibility

We explicitly separate:

- **Mechanism (router core)**: matching, stabilized `SearchMap`, transition snapshots/events, and a
  stable hook execution order.
- **Policy (app/ecosystem)**:
  - auth/permission gating
  - redirects
  - route-scoped loading/prefetch strategies
  - cancellation decisions for rapid navigation

### 2) Route hooks are defined in two phases

#### 2.1 `before_load` (route-scoped policy, synchronous in v1)

Purpose: route-level middleware that can *block* or *redirect* navigation after the router has
computed a validated match chain.

Contract:

- Evaluated root -> leaf (match chain order).
- Receives:
  - `from` and `to` canonical locations
  - the full match chain (including stabilized search)
  - a stable transition cause (`Navigate`/`Redirect`/`Sync`)
- Returns:
  - `Allow`, `Block(reason)`, or `Redirect(action, to)`

Notes:

- This is distinct from global guards (ADR 1168): global guards are coarse-grained policy; route
  `before_load` is match-aware and can be composed per subtree.
- Async `before_load` is deferred; v1 is sync to avoid locking into a runtime.

#### 2.2 `loader` (route-scoped prefetch planning, sync-by-default)

Purpose: expose a stable seam for “data should be warm” semantics (TanStack-style loaders), without
requiring router-core to know query result types.

Contract:

- Evaluated root -> leaf (match chain order).
- Receives the same match/transition context as `before_load`.
- Produces **prefetch intents** (not typed query keys):
  - `(namespace, canonical RouteLocation, extra_tag)` plus optional route identity.

Apps map these intents to concrete `fret-query` calls:

- select the `T` for `QueryKey<T>` (typed key)
- choose `QueryPolicy` (stale time, cancellation mode, retry policy)
- provide the fetch function (sync/async/local-async)

Router-core provides canonicalization helpers so apps can reliably build stable query keys.

### 3) Hook execution order is stable

For a navigation attempt:

1. Compute match chain + stabilized search (`validate_search`).
2. Evaluate global guard (if installed).
3. Evaluate route `before_load` root -> leaf.
4. Commit navigation (history mutation) only if allowed (adapter permitting).
5. Emit `RouterEvent::Transitioned` or `RouterEvent::Blocked`.
6. After commit, evaluate route `loader` root -> leaf to kick off prefetch intents.

Back/Forward note:

- If the adapter supports `peek`, steps 2–3 can run pre-navigation.
- If not, steps 2–3 may run post-navigation with a defined soft-block fallback (ADR 1168).

### 4) `fret-query` integration stays feature-gated

`ecosystem/fret-router` must not depend on `fret-query` by default. Any helper types/functions that
mention `QueryKey<T>` remain behind `query-integration`.

## Non-goals (v1)

- Async route hooks / suspending navigation.
- SSR, streaming, server functions (TanStack Start).
- Type-erased `QueryKey` planning inside router-core.

## Follow-ups

- Async hooks and cancellation semantics (tied to app-owned executors).
- A standardized “intent prefetch” API (hover/focus/keyboard intent).
- Optional serde snapshots for transitions/events/prefetch intents (diagnostics bundles).

## Evidence anchors (current baseline)

- Match/search stabilization: `ecosystem/fret-router/src/route_tree.rs`, `ecosystem/fret-router/src/search.rs`
- Transitions/events/guards: `ecosystem/fret-router/src/router_state.rs` (ADR 1168)
- Query helpers and planning: `ecosystem/fret-router/src/query_integration.rs` (feature `query-integration`)
- Query key conventions: `docs/query-key-conventions.md`

