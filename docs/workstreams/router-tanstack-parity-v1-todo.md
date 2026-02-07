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

- `[x]` Add `RouteTree` definition surface (builder API; macro-free).
- `[x]` Add `match_routes(RouteLocation) -> Vec<RouteMatch>` with accumulated:
  - `full_path`
  - `params`
  - raw `search` (pre-validation)
- `[x]` Add deterministic not-found handling:
  - global not-found route, and/or per-subtree not-found route
- `[x]` Add ambiguity diagnostics for overlapping route patterns.
- `[x]` Add unit tests for nested routes and not-found behavior.

## Phase 2 - Search validation and stabilization

- `[x]` Define canonical `SearchMap` representation (stable + duplicates preserved).
- `[x]` Add per-route `validate_search(parent_search, raw_search)` hook.
- `[x]` Define error handling policy:
  - “throw” vs “recover with raw search” (TanStack has both)
- `[x]` Add tests:
  - parent -> child accumulation
  - error path behavior

## Phase 3 - RouterState + history adapters

- `[x]` Add `RouterState` (location, matches, status, last_transition).
- `[x]` Define `RouterTransition` snapshot (portable; diagnostics-friendly).
- `[x]` Add `RouterUpdate` return type for `navigate` / `sync`.
- `[x]` Add `RouterEvent` queue (`Router::take_events()`).
- `[x]` Add guard contract:
  - `Push`/`Replace`: pre-guard (block/redirect)
  - `Back`/`Forward`: pre-guard when history can peek, post-guard fallback otherwise
- `[x]` Add redirect loop detection + hop limit (default 4).
- `[x]` Define `HistoryAdapter` trait and implement:
  - memory adapter (wrap `MemoryHistory`)
  - web history adapter (wrap existing `web-history`)
  - hash adapter (wrap existing `hash-routing`)
- `[x]` Add integration tests for:
  - back/forward restore
  - deep-link open -> matches computed

## Phase 4 - `fret-query` loader integration

- `[x]` Define route->query key conventions for loader-like behavior.
- `[x]` Add transition-based invalidate/prefetch planning:
  - input: `RouterTransition`
  - output: list of namespaces + keys to prefetch
- `[x]` Add route-level hook surface (ADR 1169):
  - `before_load` (per-route middleware)
  - `loader` (prefetch intents)
- `[x]` Add update-scoped helpers:
  - `Router::navigate_with_prefetch_intents`
  - `Router::sync_with_prefetch_intents`
- `[ ]` Add race/cancellation tests for rapid route changes.

## Phase 5 - App adoption

- `[x]` Add an app-level reference integration example:
  - window-scoped router state + navigation commands
  - query prefetch tied to route matches
- `[x]` Wire UI Gallery page history navigation:
  - add `Back`/`Forward` commands driven by the router history stack
