# Router UI v1 (Desktop Adoption) (Tracking)

Last updated: 2026-02-08

This file tracks concrete work for:

- `docs/workstreams/router-ui-v1.md`

Status legend:

- `[ ]` not started
- `[~]` in progress
- `[x]` done

## Phase 0 - Decisions and scope

- `[ ]` Confirm crate location and dependencies:
  - new crate `ecosystem/fret-router-ui`
  - depends on `fret-router` + `fret-ui`
  - optional `fret-query` integration via feature

## Phase 1 - Store + snapshot

- `[ ]` Add `RouterUiSnapshot<R>` (portable struct for UI observation).
- `[ ]` Add `RouterUiStore<R, H>`:
  - owns `Router<R, H>`
  - owns `Model<RouterUiSnapshot<R>>`
  - exposes `navigate_*` / `sync_*` returning update-scoped intents
- `[ ]` Add unit tests:
  - model snapshot updates when router changes
  - init prefetch (`init_with_prefetch_intents`) updates snapshot/intents as expected

## Phase 2 - Outlet

- `[ ]` Add `RouterOutlet` element:
  - renders by leaf route id (match chain)
  - supports a `NotFound` fallback
- `[ ]` Add diagnostics hooks:
  - optional `test_id`
  - surface last transition for debug panels

## Phase 3 - Link helpers (desktop)

- `[ ]` Add `RouterLink` element:
  - computes `href` using `Router::href_to(...)`
  - on press, performs guard-aware navigation
- `[ ]` Add optional context menu actions:
  - copy link
  - open in new window (app-owned policy)

## Phase 4 - App adoption

- `[ ]` Adopt in one desktop app:
  - show match-driven outlet rendering
  - show typed navigation via `navigate_to_*` + typed search helpers
- `[ ]` Add a `fretboard diag` script for a basic navigation flow.

