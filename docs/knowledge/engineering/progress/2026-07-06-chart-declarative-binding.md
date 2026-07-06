---
type: Work Progress
title: Chart declarative demo binding cleanup
timestamp: 2026-07-05T23:25:36Z
git_branch: main
tags: fret,ui-framework,public-surface,chart,binding,raw-model
---

# Summary

`chart_declarative_demo`, the manual-harness `chart_demo`, `bars_demo`, `category_line_demo`, and
`horizontal_bars_demo` now store a `ChartCanvasPanelBinding` instead of exposing raw chart
engine/output models in app-facing state.

The new binding is intentionally narrow:

- `ChartCanvasPanelBinding::new(host, spec, engine)` inserts the controlled chart engine model and
  a default chart output model;
- `panel_props()` builds `ChartCanvasPanelProps` with the internal engine and output models
  attached;
- `observe_engine_paint(cx)` preserves the app-view paint dependency without teaching raw model
  reads;
- `output_untracked(...)`, `output_layout(...)`, and `output_paint(...)` keep tooltip/output reads
  on the binding surface;
- `from_model(...)` and `from_models(...)` are available only as advanced bridges for callers that
  already coordinate shared chart models;
- the manual `chart_demo` keeps its `FnDriver` shell but no longer teaches `ChartCanvasPanelProps`
  engine wiring.

# Decision

Do not hide `ChartCanvasPanelProps` from component authors. The prop record is still the right
low-level composition surface for grid views, overlay-only panels, and intentionally shared output
models. The single-chart binding is the default app surface, and linked chart demos should prefer a
named linked binding instead of re-exposing raw chart model handles.

# Verification

- `cargo nextest run -p fret-chart chart_canvas_binding_creates_props_with_engine_and_output_without_public_raw_handle --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface chart_declarative_demo_uses_app_view_imports --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface chart_demo_uses_manual_harness_chart_binding --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface bars_demo_uses_declarative_canvas_panel --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface basic_chart_demos_use_declarative_canvas_panel --no-fail-fast`

# Next

Remaining chart raw-model pressure should be split by advanced contract:

- linked chart examples should use `ChartCanvasLinkedGroupBinding` and
  `ChartCanvasLinkedPanelBinding`;
- chart stress harnesses that intentionally stay manual and diagnostic-heavy;
- linked or stress tests that intentionally exercise lower-level chart props.
