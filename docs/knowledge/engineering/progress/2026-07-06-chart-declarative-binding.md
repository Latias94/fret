---
type: Work Progress
title: Chart declarative demo binding cleanup
timestamp: 2026-07-05T23:25:36Z
git_branch: main
tags: fret,ui-framework,public-surface,chart,binding,raw-model
---

# Summary

`chart_declarative_demo` and the manual-harness `chart_demo` now store a
`ChartCanvasPanelBinding` instead of exposing a raw `Model<ChartEngine>` in app-facing state.

The new binding is intentionally narrow:

- `ChartCanvasPanelBinding::new(host, spec, engine)` inserts the controlled chart engine model;
- `panel_props()` builds `ChartCanvasPanelProps` with the internal engine model attached;
- `observe_engine_paint(cx)` preserves the app-view paint dependency without teaching raw model
  reads;
- `from_model(...)` is available only as an advanced bridge for callers that already coordinate a
  shared chart engine;
- the manual `chart_demo` keeps its `FnDriver` shell but no longer teaches `ChartCanvasPanelProps`
  engine wiring.

# Decision

Do not hide `ChartCanvasPanelProps` from component authors or linked-chart demos. The prop record is
still the right low-level composition surface for output models, linked brushes, axis pointers,
domain windows, grid views, and overlay-only panels. The binding is the default single-chart app
surface, not a replacement for advanced chart coordination.

# Verification

- `cargo nextest run -p fret-chart chart_canvas_binding_creates_props_with_engine_without_public_raw_handle --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface chart_declarative_demo_uses_app_view_imports --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface chart_demo_uses_manual_harness_chart_binding --no-fail-fast`

# Next

Remaining chart raw-model pressure should be split by contract:

- chart output/linking bindings for linked or multi-grid demos;
- chart stress harnesses that intentionally stay manual and diagnostic-heavy;
- histogram/bars/other plot-family bindings where the app-facing state/output contract differs from
  a line plot.
