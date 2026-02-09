---
title: Container Queries (v1)
status: draft
date: 2026-02-09
scope: crates/fret-ui (mechanism), ecosystem/fret-ui-kit (policy), ecosystem/fret-ui-shadcn (recipes)
---

# Container Queries (v1) — Workstream

This workstream implements **container queries** for editor-grade Fret UIs:

- Runtime provides a mechanism to read **committed container bounds** (frame-lagged layout queries).
- Ecosystem provides typed helpers (breakpoints + hysteresis) and migrates recipes away from
  viewport-width approximations.

Contract source of truth:

- ADR 1170: `docs/adr/1170-container-queries-and-frame-lagged-layout-queries-v1.md`

## Why this matters (editor-grade rationale)

Viewport breakpoints (`md:`) are not sufficient once docking/panels become the default:

- Panels resize independently of the window.
- Components must adapt to the width of their local container, not the global window.
- Hard-coded viewport thresholds in component code will drift and become unmaintainable.

## Layering (non-negotiable)

- `crates/fret-ui`: mechanism only (query region identity + bounds snapshot + invalidation).
- `ecosystem/fret-ui-kit`: typed policy helpers (breakpoints, hysteresis, recommended API).
- `ecosystem/fret-ui-shadcn`: recipes that match upstream container-query outcomes.

## Deliverables

- A minimal runtime contract (mechanism) that is stable and portable (native + wasm).
- A small typed policy surface in `fret-ui-kit` that avoids oscillation (hysteresis).
- Migration of at least one "known offender" recipe (Field responsive orientation) to the container
  query mechanism, plus a regression gate.

## Tracking

- Milestones: `docs/workstreams/container-queries-v1-milestones.md`
- TODO list: `docs/workstreams/container-queries-v1-todo.md`

