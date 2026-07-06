---
type: Work Progress
title: Chart advanced surface policy classification
timestamp: 2026-07-06T00:00:00Z
git_branch: chore/chart-advanced-surface-policy
tags: fret,ui-framework,public-surface,chart,source-policy,raw-model
---

# Summary

Advanced chart demos are covered by `tools/check_surface_policy.py` instead of only by
chart-specific source-shape tests.

The classification is intentionally split by contract:

- `echarts_demo.rs` is a comparison/adapter smoke surface, but its chart engine wiring now goes
  through `ChartCanvasPanelBinding`;
- `echarts_multi_grid_demo.rs` is an advanced manual runner surface whose shared chart engine,
  per-grid panels, and overlay-only panel now go through `ChartCanvasMultiGridBinding`;
- `chart_multi_axis_demo.rs` is an advanced linked-chart coordination proof whose shared output,
  brush, axis-pointer, and domain-window state now goes through linked chart bindings;
- `chart_stress_demo.rs` is an internal perf/stress harness whose chart engine now goes through
  `ChartCanvasPanelBinding`.

# Decision

The linked and stress chart contracts are now named. `chart_multi_axis_demo.rs` uses
`ChartCanvasLinkedGroupBinding`, `ChartCanvasLinkedPanelBinding`, and
`ChartCanvasLinkedStateBinding` for linked coordination. `chart_stress_demo.rs` uses
`ChartCanvasPanelBinding::read_engine(...)` / `update_engine(...)` for perf harness reads and
mutations without exposing a raw chart engine model.

The source-policy gate keeps these decisions explicit: raw seams are allowed only while they are
listed in each surface record, and the gate fails when a listed seam becomes unused.
`echarts_demo.rs` proved the shrink path for single adapter smoke charts;
`echarts_multi_grid_demo.rs` proves the separate multi-grid binding path; linked and stress charts
now prove the dedicated coordination and perf-harness binding paths. Remaining chart raw seams
should be runner/bootstrap or genuinely custom composition seams, not default chart model plumbing.

# Verification

- `python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
- `python3 -m unittest tools.test_check_surface_policy`
- `python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples --test echarts_demo_surface echarts_demo_uses_chart_binding_for_adapter_smoke --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface echarts_multi_grid_demo_uses_declarative_grid_panels_and_overlay --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface chart_multi_axis_demo_uses_declarative_canvas_panel_with_linked_inputs --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface chart_stress_demo_uses_declarative_canvas_panel --no-fail-fast`

# Next

The next chart cleanup should continue retiring app-facing raw chart seams only when a narrow
binding preserves the real coordination semantics. Keep raw `ChartCanvasPanelProps` for component
tests and true custom composition, not as the default app/example authoring path.
