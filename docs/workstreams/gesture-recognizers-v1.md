---
title: Gesture Recognizers (v1)
status: draft
date: 2026-02-11
scope: ecosystem/fret-ui-kit gesture policies
---

# Gesture Recognizers (v1) — Workstream

This workstream introduces a small, reusable gesture recognizer layer in `ecosystem/fret-ui-kit`,
targeting mobile (touch-first) and touchpad-heavy desktop scenarios, without pushing policy into
`crates/fret-ui` (ADR 0066).

Initial scope is intentionally minimal:

- `Pan` (drag) recognition for scroll surfaces.
- A stable “winner” rule for common conflicts:
  - tap/press vs pan-to-scroll (threshold-based).

## Contract anchors

- Runtime stays mechanism-only: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Pointer capability gating: `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`

## Tracking

- Milestones: `docs/workstreams/gesture-recognizers-v1-milestones.md`
- TODO list: `docs/workstreams/gesture-recognizers-v1-todo.md`

