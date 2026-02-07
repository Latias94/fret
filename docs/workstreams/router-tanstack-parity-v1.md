# Router (TanStack Parity) v1 (Ecosystem Workstream)

Status: Draft (design targets; ADRs remain the source of truth)

Related workstreams:

- `docs/workstreams/router-v1.md` (baseline helpers + Web adapters)
- `docs/workstreams/query-lifecycle-v1.md` (loader/prefetch backend: `fret-query`)
- `docs/workstreams/state-management-v1.md` (app-owned state + selectors)

References:

- TanStack Router concepts (in-repo snapshot): `repo-ref/router`
- ADR 1168: `docs/adr/1168-router-transitions-and-guards-v1.md`
- ADR 1169: `docs/adr/1169-router-route-hooks-and-query-seam-v1.md`

## Why this exists

`ecosystem/fret-router` v1 is intentionally lightweight: URL parsing, path patterns, history helpers,
and optional `fret-query` integration primitives.

What Fret does not yet have is a single, portable routing *contract* that scales from demos to
editor-grade apps:

- nested route matching (root -> leaf match chain)
- per-route search validation + stabilization (child can depend on parent)
- route transition causes and lifecycle (push/replace/back/forward/redirect/guard)
- first-class prefetch/load integration with `fret-query`

This workstream defines a TanStack-inspired, Rust-native router core while keeping policy heavy
behavior in ecosystem/app layers.

## Goals

- Provide a match-chain router (`Vec<RouteMatch>`) instead of a single resolved route.
- Make search/query validation a first-class, per-route opt-in contract (progressively typed).
- Standardize route transitions and expose a stable transition snapshot for diagnostics.
- Integrate route changes with `fret-query` via a small, explicit adapter seam.
- Support multi-window apps: shared route definitions, window-scoped router instances.

## Non-goals (v1)

- SSR/streaming, server functions, file-based route generation.
- Proc-macro route DSL (defer; builders first).
- Replacing app-owned state (`Model<T>`) with a global reactive graph.

## Recommended decisions (to avoid future big rewrites)

### 1) Window-scoped router instances (recommended)

Use:

- shared `RouteTree` (definition) across the process
- one `RouterState` + `HistoryAdapter` per window

Rationale:

- editor-grade apps often have per-window navigation state, deep links, and history stacks
- avoids cross-window coupling and makes diagnostics/repro bundles window-addressable
- keeps the “shared definitions” benefit without global mutable routing state

### 2) Search typing strategy: progressive typing (recommended)

Start with:

- a canonical `SearchMap` representation (stable ordering, duplicate keys preserved)
- per-route `validate_search` hooks that return either:
  - a validated/stabilized `SearchMap`, or
  - a route-specific typed view wrapper (manual impl)

Then optionally add:

- `serde`-based typed search decode behind a feature (`serde-query`)
- macro helpers only after real-world boilerplate evidence (at least 2 apps)

Rationale:

- avoids blocking on perfect schema story
- keeps runtime portable (native + wasm)
- allows incremental adoption route-by-route

### 3) Loader/prefetch backend: `fret-query` as the primary integration (recommended)

Treat `fret-query` as the default cache/loader backend and model TanStack-style loader semantics as:

- route -> query keys (namespaced, canonical location)
  - router transition -> invalidate/prefetch plan
  - optional cancellation hooks for rapid navigation (future phase)

Keep a small adapter seam so tests can run with a mock client.

Note: query key canonicalization should **ignore URL fragments** by default. Fragments are typically
view-local UI state (scroll anchors, tabs) and should not invalidate loader data.

## Proposed architecture (v1 targets)

### Core types (portable)

- `RouteLocation` (already exists): `{ path, query, fragment }`
- `RouteTree` (new): nested route definitions (rooted)
- `RouteMatch` (new): one segment in the match chain
  - `route_id`
  - `full_path` (accumulated)
  - `params` (accumulated)
  - `search` (accumulated, validated)
- `RouterState` (new): `{ location, matches, status, last_transition }`

### Route matching

Target behavior:

- route matching returns a chain (root -> leaf)
- prefer specificity over insertion order
- explicit “not found” handling (global or per-subtree)

### Search validation and stabilization

Target behavior (TanStack-aligned):

- match chain builds from root to leaf
- each route can validate/augment search
- child route sees parent-validated search
- canonicalization is deterministic (important for query keys and caching)

### Transitions

Standardize a transition snapshot (portable; diagnostics-friendly):

- `cause`: `RouterTransitionCause` (`Navigate { action }`, `Redirect { action }`, `Sync`)
- `from` / `to`: canonical locations
- `redirect_chain`: attempted locations (0..N, excludes the final `to`; capped with a hop limit)
- `blocked_by`: optional guard reason
- `RouterUpdate`: `navigate` / `sync` returns a structured “changed vs no-op” result
- `RouterEvent`: a deterministic event stream (`Router::take_events()`) for diagnostics and tests
- `guard`: optional app/ecosystem policy hook:
  - `Push`/`Replace`: pre-guard (can `Allow`/`Block`/`Redirect`)
  - `Back`/`Forward`: pre-guard if the `HistoryAdapter` can `peek`; otherwise post-guard with a
    soft-block fallback (`Replace(from)`)

### History adapters

Unify under a trait:

- `MemoryHistory` (portable baseline)
- `WebHistoryAdapter` (pushState/replaceState + popstate)
- `HashHistoryAdapter` (hashchange + `#/...`)

## Phases and deliverables

### Phase 1 - Match chain + specificity diagnostics

- Add `RouteTree` and `match_routes(location) -> Vec<RouteMatch>`.
- Add route ambiguity diagnostics (detect overlapping patterns).
- Keep `RouteTable` for single-route utilities; do not break v1 callers.

### Phase 2 - Search validation

- Introduce `SearchMap` and per-route `validate_search` hooks.
- Add a lightweight “build location” path (TanStack’s `buildLocation` equivalent).

### Phase 3 - RouterState + adapters

- Add `RouterState` that reacts to `HistoryAdapter` events.
- Provide explicit APIs for `navigate`, `replace`, `back`, `forward`.

### Phase 4 - `fret-query` loader integration

- Provide a route->query key convention and invalidate/prefetch planning from transitions.
- Add race/cancellation tests for rapid route changes.

## Evidence anchors (current baseline)

- `ecosystem/fret-router/src/path.rs` (path patterns, specificity-first resolution)
- `ecosystem/fret-router/src/location.rs` (canonical location parse/format)
- `ecosystem/fret-router/src/history.rs` (portable history)
- `ecosystem/fret-router/src/web.rs` + `ecosystem/fret-router/tests/web_wasm.rs` (web adapters)
- `ecosystem/fret-router/src/query_integration.rs` (namespace invalidation planning + keying helpers)
