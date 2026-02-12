---
title: UI Performance (Windows RTX 4090) - Smoothness Closure v1 (TODO)
status: draft
date: 2026-02-12
scope: performance, regression-gates, windows
---

# UI Performance (Windows RTX 4090) - Smoothness Closure (v1) — TODO

Workstream note:

- Plan: `docs/workstreams/ui-perf-windows-rtx4090-smoothness-v1.md`
- Milestones: `docs/workstreams/ui-perf-windows-rtx4090-smoothness-v1-milestones.md`

---

## P0 - Measurement hygiene (must do first)

- [ ] Create a clean worktree for perf runs (`fret-worktrees/`) so results are commit-addressable.
- [ ] Capture `ui-gallery-steady` + `ui-resize-probes` + `ui-code-editor-resize-probes` in that clean tree.
- [ ] Record exact machine profile (Windows build, monitor refresh, scaling, NVIDIA driver).

## P0 - Gate stability and script robustness

- [ ] Ensure `ui-gallery-steady` scripts remain stable under `--reuse-launch` on Windows.
- [ ] Add/keep `click_stable` where navigation targets can shift due to scroll/transform-only updates.

## P1 - Resize tail latency (layout dominated)

- [ ] For `ui-gallery-window-resize-stress-steady`, reduce tail frames where `layout_request_build_roots_time_us` spikes.
- [ ] For `ui-gallery-window-resize-drag-jitter-steady`, reduce tail frames where `layout_time_us` dominates.
- [ ] Validate with `ui-resize-probes --attempts 3` and keep worst bundles explainable.

## P1 - Steady suite tail spikes

- [ ] Investigate `ui-gallery-view-cache-toggle-perf-steady` tail spikes (layout-heavy).
- [ ] Investigate `ui-gallery-overlay-torture-steady` tail spikes (layout-heavy).
- [ ] For each script, produce a “hitch class” note (top 3 metrics + top 3 hotspots from `diag stats`).

## P1.5 - Diagnostics semantics policy (keep perf runs cheap)

- [x] Stop forcing `UiTree::request_semantics_snapshot()` every frame in gallery/bootstrap render drivers; only request
  semantics snapshots when diagnostics state wants them.
- [ ] Tighten the “schema v2 intent step” semantics policy so multi-frame steps only request semantics when selector
  resolution is needed (avoid semantics refresh churn during long drags).

## P2 - CPU vs GPU separation (only after CPU tail is clear)

- [ ] Run worst-case scripts with `FRET_DIAG_RENDERER_PERF=1` to check whether any tail spikes correlate with renderer churn.
- [ ] If GPU signals look suspicious, capture with RenderDoc (`docs/renderdoc-inspection.md`) on the worst run.

## P2 - Pointer-move / hit-test gates (optional on Windows)

- [ ] Verify harness-only mode for hit-test torture and ensure the root `test_id` exists on the harness page.
- [ ] Establish a Windows pointer-move gate (dispatch/hit-test thresholds) if it is stable and useful.
