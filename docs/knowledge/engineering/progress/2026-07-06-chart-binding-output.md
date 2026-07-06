---
type: Work Progress
title: Chart binding-owned output channel
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/chart-binding-output
tags: fret,ui-framework,public-surface,chart,binding,output,raw-model
---

# Summary

`ChartCanvasPanelBinding` now owns the default single-chart output channel alongside the controlled
chart engine model.

The app-facing contract mirrors the plot bindings:

- `new(host, spec, engine)` inserts the chart engine and a default `ChartCanvasOutput`;
- `panel_props()` wires both models into `ChartCanvasPanelProps`;
- `output_untracked(...)` supports event-handler logging without exposing raw output models;
- `output_layout(...)` and `output_paint(...)` support tracked reads from render/layout contexts;
- `from_models(...)` is the advanced bridge for callers that already share engine/output models.

# Decision

Keep binding-owned output limited to the default single-chart app/cookbook surface. Multi-grid
charts use `ChartCanvasMultiGridBinding`, which deliberately does not allocate a default output
model because multiple grid panels plus one overlay panel would race to publish a single current
output. Linked charts, explicit output sharing, and stress harnesses should continue to use raw
`ChartCanvasPanelProps` until their coordination contracts are named.

# Verification

- `cargo nextest run -p fret-chart chart_canvas_binding_creates_props_with_engine_and_output_without_public_raw_handle --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface bars_demo_uses_declarative_canvas_panel --no-fail-fast`

# Next

`echarts_demo.rs` now uses `ChartCanvasPanelBinding` for adapter smoke charts, and
`echarts_multi_grid_demo.rs` uses `ChartCanvasMultiGridBinding` for shared-engine grid views plus
overlay-only panels. Name explicit contracts before migrating linked or stress demos; those
surfaces still use raw `ChartCanvasPanelProps` intentionally because they exercise shared output,
linkage state, or perf harness state.
