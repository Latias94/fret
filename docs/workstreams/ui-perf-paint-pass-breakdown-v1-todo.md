---
title: UI Performance: Paint Pass Breakdown v1 (TODO)
status: draft
date: 2026-02-05
scope: diagnostics, paint, view-cache, paint-cache, profiling
---

# UI Performance: Paint Pass Breakdown v1 (TODO)

This file tracks milestones and concrete tasks for:

- `docs/workstreams/ui-perf-paint-pass-breakdown-v1.md`

Conventions:

- “Perf gate” items should land with a runnable `fretboard diag perf` command + a bundle path.
- “Fearless refactor” items should include: (1) perf evidence, (2) correctness evidence, (3) rollback plan.

## Milestones

### P0: Explain the paint pass (make paint hotspots visible)

- [x] Export coarse paint-cache attribution (replay time, bounds translation time, element visual-bounds recording time).
  - Evidence: `feat(diag): add paint pass breakdown metrics` (commit `f2bee87a`).
- [x] Add initial paint-phase micro timers for “paint-all” plumbing (see workstream doc section 1):
  - window snapshot plumbing performed during paint,
  - scroll-handle invalidation before paint-cache replay,
  - layer root enumeration,
  - paint observation collapse after paint.
  - Evidence: `feat(diag): add paint micro-breakdown timers` (commit `b20a1280`).
- [ ] Add paint-phase micro timers for the remaining dominant candidates:
  - per-node traversal overhead on stable frames (cache hit vs miss),
  - widget `paint()` time,
  - observation bookkeeping (recording/merging observed globals/models during paint).
- [x] Update `fretboard diag stats` + `--json` output to include the initial paint micro timers.
  - Evidence: `feat(diag): add paint micro-breakdown timers` (commit `b20a1280`).
- [ ] Record at least 3 “stable but paint-heavy” evidence bundles (menubar, overlay torture, chrome torture) and
  summarize the dominant paint sub-slice for each.

### P1: Reduce stable-frame paint overhead (first real win)

- [ ] Pick one evidence bundle where paint is dominated by a single sub-slice and fix it.
- [ ] Validate against `ui-gallery-steady` baseline (repeat=7) and record delta in the perf log.
- [ ] Ensure view-cache correctness remains unchanged:
  - `cargo test -p fret-ui` (or nextest equivalent) remains green,
  - targeted scripted probes still pass (`fretboard diag repro ...` smoke runs).

### P2: GPUI-aligned caching surface (optional, contract-heavy)

- [ ] Decide whether “cached view can skip widget paint without notify” should become an explicit contract (ADR).
- [ ] Prototype one of:
  - per-view paint replay that bypasses per-node traversal on stable frames, or
  - renderer-friendly replay primitives (encoded display list chunks).
- [ ] Add a gate that fails when stable-frame paint cost regresses on chrome/menus (Tier B candidate).
