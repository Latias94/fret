# Router (TanStack Parity) v1 (Tracking)

Last updated: 2026-02-07

This file tracks concrete work for:

- `docs/workstreams/router-tanstack-parity-v1.md`

Status legend:

- `[ ]` not started
- `[~]` in progress
- `[x]` done

## Phase 0 - Decisions and scope

- `[ ]` Confirm router instance ownership model:
  - shared `RouteTree`, per-window `RouterState` + history (recommended)
- `[ ]` Confirm search typing strategy:
  - progressive typing (recommended)
- `[ ]` Confirm loader backend:
  - `fret-query` primary integration (recommended)

## Phase 1 - Match chain core

- `[ ]` Add `RouteTree` definition surface (builder API; macro-free).
- `[ ]` Add `match_routes(RouteLocation) -> Vec<RouteMatch>` with accumulated:
  - `full_path`
  - `params`
  - raw `search` (pre-validation)
- `[ ]` Add deterministic not-found handling:
  - global not-found route, and/or per-subtree not-found route
- `[ ]` Add ambiguity diagnostics for overlapping route patterns.
- `[ ]` Add unit tests for nested routes and not-found behavior.

## Phase 2 - Search validation and stabilization

- `[ ]` Define canonical `SearchMap` representation (stable + duplicates preserved).
- `[ ]` Add per-route `validate_search(parent_search, raw_search)` hook.
- `[ ]` Define error handling policy:
  - “throw” vs “recover with raw search” (TanStack has both)
- `[ ]` Add tests:
  - parent -> child accumulation
  - error path behavior

## Phase 3 - RouterState + history adapters

- `[ ]` Add `RouterState` (location, matches, status, last_transition).
- `[ ]` Define `RouterTransition` snapshot (portable, serializable).
- `[ ]` Define `HistoryAdapter` trait and implement:
  - memory adapter (wrap `MemoryHistory`)
  - web history adapter (wrap existing `web-history`)
  - hash adapter (wrap existing `hash-routing`)
- `[ ]` Add integration tests for:
  - back/forward restore
  - deep-link open -> matches computed

## Phase 4 - `fret-query` loader integration

- `[ ]` Define route->query key conventions for loader-like behavior.
- `[ ]` Add transition-based invalidate/prefetch planning:
  - input: `RouterTransition`
  - output: list of namespaces + keys to prefetch
- `[ ]` Add race/cancellation tests for rapid route changes.

## Phase 5 - App adoption

- `[ ]` Add an app-level reference integration example:
  - window-scoped router state + navigation commands
  - query prefetch tied to route matches

