---
type: Work Progress
title: Histogram plot binding cleanup
timestamp: 2026-07-05T23:42:37Z
git_branch: main
tags: fret,ui-framework,public-surface,plot,histogram,binding,raw-model
---

# Summary

`histogram_demo` now stores a `HistogramPlotPanelBinding` instead of separate raw
`Model<HistogramPlotModel>`, `Model<PlotState>`, and `Model<PlotOutput>` handles.

The binding work also refactors `fret-plot`'s app-facing bindings around a private
`PlotPanelBindingCore<M>` so line and histogram bindings share the same model/state/output ownership
and output-read helpers.

# Decision

Keep the generic binding core private. The public API stays family-specific:

- `LinePlotPanelBinding` for line plots;
- `HistogramPlotPanelBinding` for histogram plots.

This avoids a leaky public generic abstraction while still preventing duplicated output/state
plumbing inside `fret-plot`.

# Verification

- `cargo nextest run -p fret-plot line_plot_binding_creates_props_with_state_and_output_without_public_raw_handles line_plot_binding_reads_output_without_exposing_output_model_handle histogram_plot_binding_creates_props_with_state_and_output_without_public_raw_handles histogram_plot_binding_reads_output_without_exposing_output_model_handle --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_plot_demos_surface histogram_demo_uses_manual_harness_declarative_histogram_plot_panel --no-fail-fast`

# Next

This pattern later extended to the other app-facing plot families. Advanced line plot examples now
use `LinePlotPanelBinding::new_with_state(...)`, `update_state(...)`, `output_untracked(...)`,
`linked_member()`, and `update_model(...)` for overlay, linked-cursor, drag, and stress behavior
without exposing raw runtime model handles to app examples.
