---
type: Work Progress
title: Single-panel plot binding cleanup
timestamp: 2026-07-06T00:46:00Z
git_branch: main
tags: fret,ui-framework,public-surface,plot,binding,raw-model
---

# Summary

The remaining first-contact single-panel plot demos now use family-specific app-facing bindings:

- `AreaPlotPanelBinding` in `area_demo`;
- `ShadedPlotPanelBinding` in `shaded_demo`;
- `CandlestickPlotPanelBinding` in `candlestick_demo`;
- `HeatmapPlotPanelBinding` in `heatmap_demo`;
- `Histogram2DPlotPanelBinding` in `histogram2d_demo`.

Each binding owns the plot model plus `PlotState` and `PlotOutput`, exposes `panel_props()` for
declarative rendering, and exposes `output_untracked(...)` where event logging needs a read.

# Decision

Treat single-panel cookbook demos as app-facing surfaces. Raw `*PlotPanelProps::new(Model<_>)`
remains the component-author and advanced-composition surface, not the default authoring pattern.

Do not mechanically migrate demos that mutate plot state, install overlays, or link multiple
panels. This doc was written before the advanced line binding surface landed; those cases later
gained named APIs on `LinePlotPanelBinding` for initial state, state mutation, output reads, linked
members, and controlled model mutation without exposing raw runtime handles to app examples.

# Verification

- `cargo nextest run -p fret-plot area_plot_binding_creates_props_with_state_and_output_without_public_raw_handles area_plot_binding_reads_output_without_exposing_output_model_handle shaded_plot_binding_creates_props_with_state_and_output_without_public_raw_handles shaded_plot_binding_reads_output_without_exposing_output_model_handle candlestick_plot_binding_creates_props_with_state_and_output_without_public_raw_handles candlestick_plot_binding_reads_output_without_exposing_output_model_handle heatmap_plot_binding_creates_props_with_state_and_output_without_public_raw_handles heatmap_plot_binding_reads_output_without_exposing_output_model_handle histogram2d_plot_binding_creates_props_with_state_and_output_without_public_raw_handles histogram2d_plot_binding_reads_output_without_exposing_output_model_handle --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_plot_demos_surface area_demo_uses_manual_harness_declarative_area_plot_panel shaded_demo_uses_manual_harness_declarative_shaded_plot_panel candlestick_demo_uses_manual_harness_declarative_candlestick_plot_panel heatmap_demo_uses_manual_harness_declarative_heatmap_plot_panel histogram2d_demo_uses_manual_harness_declarative_histogram2d_plot_panel --no-fail-fast`

# Next

The named contracts for advanced plot overlays, linked panels, and stress mutation later landed on
`LinePlotPanelBinding`. Future plot cleanup should keep raw props in component tests and true custom
composition paths, while app examples should prefer a family-specific binding first.
