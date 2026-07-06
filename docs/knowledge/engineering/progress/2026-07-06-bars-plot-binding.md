---
type: Work Progress
title: Bars plot binding cleanup
timestamp: 2026-07-06T00:28:00Z
git_branch: main
tags: fret,ui-framework,public-surface,plot,bars,binding,raw-model
---

# Summary

`grouped_bars_demo` and `stacked_bars_demo` now store a `BarsPlotPanelBinding` instead of separate
raw `Model<BarsPlotModel>`, `Model<PlotState>`, and `Model<PlotOutput>` handles.

This keeps both bars demos in the default app-facing plot surface:

- `BarsPlotPanelBinding::new(host, model)` owns the bars model, `PlotState`, and `PlotOutput`;
- `panel_props()` builds `BarsPlotPanelProps` with the internal state/output models attached;
- `output_untracked(...)` keeps event logging from depending on a raw output model handle.

# Decision

Use one `BarsPlotPanelBinding` for grouped and stacked bars because both are construction policies
over the same `BarsPlotModel`. Do not introduce grouped/stacked-specific binding types unless their
runtime contracts diverge beyond model construction.

# Verification

- `cargo nextest run -p fret-plot bars_plot_binding_creates_props_with_state_and_output_without_public_raw_handles bars_plot_binding_reads_output_without_exposing_output_model_handle --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_plot_demos_surface grouped_bars_demo_uses_manual_harness_declarative_bars_plot_panel stacked_bars_demo_uses_manual_harness_declarative_bars_plot_panel --no-fail-fast`

# Next

Evaluate candlestick, area, shaded, heatmap, and histogram2d demos with the same rule: add
app-facing bindings for first-contact examples, but keep raw prop records where the demo is proving
advanced composition or linked plot contracts.
