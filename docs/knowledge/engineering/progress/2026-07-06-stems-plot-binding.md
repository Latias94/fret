---
type: Work Progress
title: Stems plot binding cleanup
timestamp: 2026-07-05T23:58:30Z
git_branch: main
tags: fret,ui-framework,public-surface,plot,stems,binding,raw-model
---

# Summary

`stems_demo` now stores a `StemsPlotPanelBinding` instead of separate raw
`Model<StemsPlotModel>`, `Model<PlotState>`, and `Model<PlotOutput>` handles.

This extends the `fret-plot` app-facing binding pattern beyond line and histogram panels:

- `StemsPlotPanelBinding::new(host, model)` owns the plot model, `PlotState`, and `PlotOutput`;
- `panel_props()` builds `StemsPlotPanelProps` with the internal state/output models attached;
- `output_untracked(...)` keeps event logging from depending on a raw output model handle.

# Decision

Keep `StemsPlotPanelBinding` as a family-specific public type backed by the private
`PlotPanelBindingCore<M>`. Do not expose the generic core publicly; app authors should see named
plot-family bindings, while component authors can still use raw prop records for advanced
composition.

# Verification

- `cargo nextest run -p fret-plot line_plot_binding_creates_props_with_state_and_output_without_public_raw_handles line_plot_binding_reads_output_without_exposing_output_model_handle histogram_plot_binding_creates_props_with_state_and_output_without_public_raw_handles histogram_plot_binding_reads_output_without_exposing_output_model_handle stems_plot_binding_creates_props_with_state_and_output_without_public_raw_handles stems_plot_binding_reads_output_without_exposing_output_model_handle --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_plot_demos_surface stems_demo_uses_manual_harness_declarative_stems_plot_panel --no-fail-fast`

# Next

Evaluate `error_bars_demo` and bar-family demos next. They can follow this pattern only when they
are intended to be app-facing examples rather than advanced plot composition/reference surfaces.
