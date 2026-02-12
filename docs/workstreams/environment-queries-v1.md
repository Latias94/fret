---
title: Environment Queries (v1)
status: draft
date: 2026-02-09
scope: crates/fret-ui (mechanism), ecosystem/fret-ui-kit (policy), ecosystem/* (recipe adoption)
---

# Environment Queries (v1) — Workstream

This workstream locks and implements **environment queries** for Fret UIs:

- Runtime provides a mechanism to read a **committed per-window environment snapshot** (viewport
  bounds, input capabilities, user preference hints).
- Ecosystem provides typed helpers so recipes stop hard-coding `cx.bounds` magic numbers for
  device/viewport breakpoints.

Contract source of truth:

- ADR 0232: `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`

## Why this matters

Container queries (ADR 0231) are necessary but not sufficient:

- Some responsiveness is **device/viewport-driven** (e.g. “mobile shell” patterns like
  Drawer-vs-Popover).
- Interaction affordances depend on **pointer capabilities** (hover vs touch-first).
- Future mobile targets need **safe-area** and coarse-pointer semantics.

Without a contract, these decisions drift into ad-hoc code and are hard to audit or migrate.

## Layering (non-negotiable)

- `crates/fret-ui`: mechanism only (snapshot storage + dependency tracking + diagnostics export).
- `ecosystem/fret-ui-kit`: policy helpers (breakpoints, “mobile shell” gates, capability-based
  defaults).
- `ecosystem/*`: recipes use helpers and avoid raw `cx.bounds` thresholds.

## Tracking

- Milestones: `docs/workstreams/environment-queries-v1-milestones.md`
- TODO list: `docs/workstreams/environment-queries-v1-todo.md`

