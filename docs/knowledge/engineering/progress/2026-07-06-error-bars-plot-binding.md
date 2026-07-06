---
type: Work Progress
title: Error-bars plot binding cleanup
timestamp: 2026-07-06T00:12:00Z
git_branch: main
tags: fret,ui-framework,public-surface,plot,error-bars,binding,raw-model
---

# Summary

`error_bars_demo` now stores an `ErrorBarsPlotPanelBinding` instead of separate raw
`Model<ErrorBarsPlotModel>`, `Model<PlotState>`, and `Model<PlotOutput>` handles.

This keeps the demo in the default app-facing plot surface:

- `ErrorBarsPlotPanelBinding::new(host, model)` owns the plot model, `PlotState`, and `PlotOutput`;
- `panel_props()` builds `ErrorBarsPlotPanelProps` with the internal state/output models attached;
- `output_untracked(...)` keeps event logging from depending on a raw output model handle.

# Decision

Keep every public plot binding family-specific. The shared implementation is now a private
`define_plot_panel_binding!` macro over `PlotPanelBindingCore<M>`, so adding a new app-facing plot
family no longer requires copying the same model/state/output wrapper methods. Do not expose a
generic public binding; app authors should see named plot-family bindings, while component authors
can still use raw prop records for advanced composition.

# Verification

- `cargo nextest run -p fret-plot line_plot_binding_creates_props_with_state_and_output_without_public_raw_handles line_plot_binding_reads_output_without_exposing_output_model_handle histogram_plot_binding_creates_props_with_state_and_output_without_public_raw_handles histogram_plot_binding_reads_output_without_exposing_output_model_handle stems_plot_binding_creates_props_with_state_and_output_without_public_raw_handles stems_plot_binding_reads_output_without_exposing_output_model_handle error_bars_plot_binding_creates_props_with_state_and_output_without_public_raw_handles error_bars_plot_binding_reads_output_without_exposing_output_model_handle --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_plot_demos_surface error_bars_demo_uses_manual_harness_declarative_error_bars_plot_panel --no-fail-fast`

# Next

Evaluate the bar-family demos next. If they are first-contact examples, add app-facing bindings;
if they intentionally demonstrate linked/advanced composition, keep raw prop records and document
that boundary explicitly.
