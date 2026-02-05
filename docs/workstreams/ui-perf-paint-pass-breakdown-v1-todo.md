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
- [x] Add paint-phase micro timers for paint-node breakdown:
  - paint-cache key computation,
  - paint-cache hit-check time (excluding replay),
  - exclusive widget `paint()` time (pauses while painting children),
  - observation recording time (`observed_in_paint` + globals).
  - Evidence: `feat(diag): add paint node breakdown timers` (commit `c512be81`).
- [x] Export top-N widget paint hotspots (exclusive time) so `paint_widget_time_us` is attributable.
  - Evidence: `feat(diag): export paint widget hotspots` (commit `e1132c95`).
  - Evidence bundle: `target/fret-diag-perf/menubar-kbd-nav.after-paint-widget-hotspots.1770292980/.../bundle.json`
- [x] Add `ElementHostWidget` paint sub-timers so we can explain the top hotspots:
  - obs-models iteration,
  - obs-globals iteration,
  - element instance lookup,
  - view-cache / frame-prep overhead (first-call per frame).
  - Evidence: `feat(diag): add host-widget paint sub-timers` (commit `188d7da1`).
  - Result (menubar steady probe): obs-models/globals/instance lookup are each O(10us) and do not explain the ~1ms+
    `ElementHostWidget` hotspots; next step is to time child traversal / bounds queries / clip setup overhead.
- [ ] Add paint-phase micro timers for the remaining dominant candidates:
  - per-node traversal overhead on stable frames (excluding widget code),
  - observation merging/collapse costs beyond the already-timed “collapse observations” step.
- [x] Update `fretboard diag stats` + `--json` output to include the initial paint micro timers.
  - Evidence: `feat(diag): add paint micro-breakdown timers` (commit `b20a1280`).
- [ ] Record at least 3 “stable but paint-heavy” evidence bundles (menubar, overlay torture, chrome torture) and
  summarize the dominant paint sub-slice for each.

### P1: Reduce stable-frame paint overhead (first real win)

- [ ] Pick one evidence bundle where paint is dominated by a single sub-slice and fix it.
- [ ] Remove per-frame allocations in the `ElementHostWidget` paint path.
  - Hypothesis: element-runtime observation accessors clone per-element dependency vectors each frame.
  - Candidate fix: replace `elements::{observed_models_for_element, observed_globals_for_element}` returning `Vec<_>`
    with a zero-allocation iterator/closure API, and update host-widget layout/measure/paint to use it.
  - Evidence target: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-05 20:03 (commit `e1132c95`).
  - Update: initial attempts (`424ca9fc`, `df5df0b7`) did not materially reduce `ElementHostWidget` paint hotspots on the
    menubar steady probe (see log entries 2026-02-05 20:28 and 20:37).
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

## Notes / experiments

- [x] A/B: relax paint-cache view-cache gating (`FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING=1`).
  - Result: increased paint-cache hits and reduced `paint_nodes_performed`, but did not materially improve
    `paint_widget_time_us` on the menubar steady probe.
  - Evidence: perf log entry 2026-02-05 19:25 (commit `f3078d25`).
