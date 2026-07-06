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

- `echarts_demo.rs` is a comparison/adapter smoke surface;
- `echarts_multi_grid_demo.rs` is an advanced multi-grid and overlay-only chart composition proof;
- `chart_multi_axis_demo.rs` is an advanced linked-chart coordination proof with shared output,
  brush, axis-pointer, and domain-window models;
- `chart_stress_demo.rs` is an internal perf/stress harness.

# Decision

Do not migrate these demos to `ChartCanvasPanelBinding` until their advanced contracts are named.
The source-policy gate now makes that decision explicit: raw seams are allowed only while they are
listed in each surface record, and the gate fails when a listed seam becomes unused.

# Verification

- `python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
- `python3 -m unittest tools.test_check_surface_policy`
- `python3 tools/check_surface_policy.py`

# Next

The next chart cleanup should be contract design, not mechanical migration: define binding surfaces
for multi-grid, linked, adapter, or stress use cases only when they can preserve the explicit
coordination semantics that the current raw props demonstrate.
