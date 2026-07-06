---
type: Work Progress
title: Chart advanced surface policy classification
timestamp: 2026-07-06T00:00:00Z
git_branch: chore/chart-advanced-surface-policy
tags: fret,ui-framework,public-surface,chart,source-policy,raw-model
---

# Summary

Remaining advanced chart demos are now covered by `tools/check_surface_policy.py` instead of only
by chart-specific source-shape tests.

The classification is intentionally split by contract:

- `echarts_demo.rs` is a comparison/adapter smoke surface, but its chart engine wiring now goes
  through `ChartCanvasPanelBinding`;
- `echarts_multi_grid_demo.rs` is an advanced manual runner surface whose shared chart engine,
  per-grid panels, and overlay-only panel now go through `ChartCanvasMultiGridBinding`;
- `chart_multi_axis_demo.rs` is an advanced linked-chart coordination proof with shared output,
  brush, axis-pointer, and domain-window models;
- `chart_stress_demo.rs` is an internal perf/stress harness.

# Decision

Do not migrate the remaining linked and stress chart demos to `ChartCanvasPanelBinding` until their
advanced contracts are named. The source-policy gate makes that decision explicit: raw seams are
allowed only while they are listed in each surface record, and the gate fails when a listed seam
becomes unused. `echarts_demo.rs` proved the shrink path for single adapter smoke charts;
`echarts_multi_grid_demo.rs` now proves the separate multi-grid binding path. Its remaining
`fret_runtime` allowance is a manual runner/bootstrap seam, not chart model plumbing.

# Verification

- `python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
- `python3 -m unittest tools.test_check_surface_policy`
- `python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples --test echarts_demo_surface echarts_demo_uses_chart_binding_for_adapter_smoke --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface echarts_multi_grid_demo_uses_declarative_grid_panels_and_overlay --no-fail-fast`

# Next

The next chart cleanup should be contract design, not mechanical migration: define binding surfaces
for linked or stress use cases only when they can preserve the explicit coordination semantics that
the current raw props demonstrate.
