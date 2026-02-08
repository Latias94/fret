# Router UI v1 (Desktop Adoption) (Tracking)

Last updated: 2026-02-08

This file tracks concrete work for:

- `docs/workstreams/router-ui-v1.md`

Status legend:

- `[ ]` not started
- `[~]` in progress
- `[x]` done

## Phase 0 - Decisions and scope

- `[x]` Confirm crate location and dependencies:
  - new crate `ecosystem/fret-router-ui`
  - depends on `fret-router` + `fret-ui`
  - optional `fret-query` integration via feature

## Phase 1 - Store + snapshot

- `[x]` Add `RouterUiSnapshot<R>` (portable struct for UI observation).
- `[x]` Add `RouterUiStore<R, H>`:
  - owns `Router<R, H>`
  - owns `Model<RouterUiSnapshot<R>>`
  - exposes `navigate_*` / `sync_*` returning update-scoped intents
- `[x]` Add unit tests:
  - model snapshot updates when router changes
  - init prefetch (`init_with_prefetch_intents`) updates snapshot/intents as expected

## Phase 2 - Outlet

- `[x]` Add match-chain helpers on `RouterUiSnapshot`:
  - `leaf_match()` / `leaf_route()`
- `[x]` Add an outlet-style helper (`router_outlet`) for reading the snapshot model with deterministic invalidation.
- `[x]` Add a `RouterOutlet` element wrapper (optional sugar):
  - renders by leaf route id (match chain)
  - supports a `NotFound` fallback
- `[x]` Add diagnostics hooks:
  - optional `test_id` (`router_outlet_with_test_id`, `RouterOutlet::test_id`, `router_link_with_test_id`)
  - last transition is surfaced via `RouterUiSnapshot::last_transition`

## Phase 3 - Link helpers (desktop)

- `[x]` Add `RouterLink` helper:
  - compute canonical `RouteLocation` + `href` (`RouterUiStore::link_to`)
  - provide desktop affordance hook (`copy_href_on_activate`)
- `[x]` Add a navigation activation hook:
  - `RouterUiStore::navigate_link_on_activate(link)` updates router + snapshot + intents
- `[x]` Add hover prefetch intent wiring:
  - `Router::prefetch_intents_for_location(...)` (router core; no navigation)
  - `RouterUiStore::prefetch_link_on_hover_change(link)` updates intents model on hover
- `[x]` Add a low-level `router_link(...)` pressable helper (no shadcn dependency)
- `[x]` Add `RouterLink` element helpers:
  - build a link via `RouterUiStore::link_to(...)` and render it as a pressable
  - on activate, performs guard-aware navigation
- `[x]` Add optional context menu action descriptors:
  - copy link
  - open in new window (app-owned policy)

## Phase 4 - App adoption

- `[x]` Adopt in one desktop app:
  - show match-driven outlet rendering
  - show typed navigation via `navigate_to_*` + typed search helpers
- `[x]` Add a `fretboard diag` script for a basic navigation flow (`tools/diag-scripts/router-query-demo-basic-nav.json`).
