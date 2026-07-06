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

Keep binding-owned output limited to the default single-chart app/cookbook surface. Linked charts,
multi-grid overlays, explicit output sharing, and stress harnesses should continue to use raw
`ChartCanvasPanelProps` until their coordination contracts are named. This avoids hiding advanced
synchronization semantics behind the first-contact binding.

# Verification

- `cargo nextest run -p fret-chart chart_canvas_binding_creates_props_with_engine_and_output_without_public_raw_handle --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface bars_demo_uses_declarative_canvas_panel --no-fail-fast`

# Next

Name explicit contracts before migrating linked, multi-grid, adapter, or stress demos. Those demos
still use raw `ChartCanvasPanelProps` intentionally because they exercise shared output, grid views,
overlay-only panels, lower-level adapter wiring, or perf harness state.
