---
type: Work Progress
title: Chart multi-grid binding
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/raw-surface-contracts
tags: fret,ui-framework,public-surface,chart,binding,raw-model
---

# Summary

`echarts_multi_grid_demo.rs` now uses `ChartCanvasMultiGridBinding` instead of storing a raw
`Model<ChartEngine>` and hand-wiring `ChartCanvasPanelProps::engine`.

The binding owns:

- one shared chart engine model;
- the caller-provided grid order;
- per-grid `ChartCanvasPanelProps` via `grid_panel_props(grid)`;
- an overlay-only panel via `overlay_panel_props()`;
- paint observation through `observe_engine_paint(...)`.

# Decision

`ChartCanvasMultiGridBinding` deliberately does not allocate or publish a default
`ChartCanvasOutput` model. Multiple grid panels plus one overlay panel would otherwise compete to
publish one current output. Linked or aggregated chart output needs a separate explicit contract.

The `echarts_multi_grid_demo.rs` source-policy record still allows `fret_runtime`, but that seam is
now only the manual runner/bootstrap `PlatformCapabilities` path. It is no longer chart engine
model plumbing.

# Verification

- `cargo nextest run -p fret-chart chart_canvas_multi_grid_binding_creates_grid_and_overlay_props_without_output_model --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface echarts_multi_grid_demo_uses_declarative_grid_panels_and_overlay --no-fail-fast`
- `python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
- `python3 tools/check_surface_policy.py`

# Next

Linked chart demos still need a dedicated contract for shared output, brush, axis-pointer, and
domain-window models. Stress chart demos should stay on their perf harness path until engine stats
and progressive rendering diagnostics have a public harness contract.
