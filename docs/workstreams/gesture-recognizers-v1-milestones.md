---
title: Gesture Recognizers (v1) — Milestones
status: draft
date: 2026-02-11
scope: ecosystem/fret-ui-kit gesture policies
---

# Gesture Recognizers (v1) — Milestones

Workstream entry:

- `docs/workstreams/gesture-recognizers-v1.md`

## M0 — Baseline pan recognizer (single pointer)

Definition of done:

- A minimal pan recognizer exists in `ecosystem/fret-ui-kit` and can be attached to a
  `PointerRegion` surface.
- The recognizer:
  - starts tracking on pointer-down (touch-first),
  - arms after a drag threshold,
  - once armed, captures pointer and emits pan deltas until up/cancel.

Evidence:

- Unit test covers threshold arming + pointer capture + delta sign conventions.

## M1 — Scroll integration

Definition of done:

- `fret-ui-shadcn::ScrollArea` uses the recognizer for touch pan scrolling.
- Tap behavior remains correct for pressables inside the scroll surface when the pan does not arm.

Evidence:

- Unit test verifies a “tap” (no drag) still bubbles to a child pressable.

## M2 — Extensions (deferred)

Candidates:

- inertial scrolling (kinetic) with reduced-motion gating,
- axis locking heuristics,
- pinch-to-zoom policies for viewport surfaces.

