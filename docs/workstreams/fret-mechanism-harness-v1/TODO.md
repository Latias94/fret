---
title: Fret Mechanism Harness v1 TODO
status: active
date: 2026-05-11
---

# TODO

- [x] Create the dedicated workstream and coverage map.
- [x] Extend `fret-mechanism-harness` with scalar mechanism metrics for non-geometry facts.
- [x] Add fixture-driven layout dirty invalidation cases for suppressed boundaries and repair paths.
- [x] Connect the checkbox "Enable notifications" underflow path as the first UI Gallery runtime
  diagnostics gate.
- [x] Run the first gate set and record actual command results in `FINDINGS_2026-05-11.md`.
- [x] If a new confirmed defect appears, fix it in the owning layer and add regression coverage.
  - Result: no new confirmed defect appeared in this slice; the known suppressed-boundary defect was
    already fixed and is now locked by fixture coverage plus diagnostics.
- [x] Pick the next slice from the largest uncovered mechanism gap.
- [x] Extend layout dirty invalidation fixtures into view-cache/root-boundary invalidation:
  retained contained relayout, scroll/direct-child dirty frontier coverage, detached dirty-cache-root
  pruning, and view-cache layout-dirty expansion attribution.
- [x] Run focused gates for the view-cache/root-boundary slice and record the results.
- [x] Add a dedicated scroll-handle invalidation fixture suite for windowed-paint cache-root dirtying,
  virtual-list window escape, revision-only baseline handling, and detached stale binding filtering.
- [x] Run focused gates for the scroll-handle window-update slice and record the results.
- [x] Promote environment-triggered cache-root invalidation into a fixture-driven declarative
  cache-hit harness using the real `WindowMetricsService` entry point.
- [x] Fix the confirmed environment sync defect exposed by the harness.
- [ ] Add a real UI Gallery scroll/virtual-list diagnostics gate once the smallest stable demo page
  and selectors are identified.
- [ ] Add UI Gallery diagnostics for runtime platform preference/environment changes once a stable
  demo page exists.
