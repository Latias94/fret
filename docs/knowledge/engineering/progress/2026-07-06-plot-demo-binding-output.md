---
type: Work Progress
title: Plot demo binding output cleanup
timestamp: 2026-07-05T23:08:46Z
git_branch: main
tags: fret,ui-framework,public-surface,plot,binding,raw-model
---

# Summary

`plot_demo` now stores a `LinePlotPanelBinding` instead of separate raw
`Model<LinePlotModel>`, `Model<PlotState>`, and `Model<PlotOutput>` handles.

The demo still uses the manual function-driver harness, but its plot state surface now follows the
same app-facing binding contract as `plot_declarative_demo`:

- initialization builds the plot through `LinePlotPanelBinding::new(app, model)`;
- render builds panel props through `plot.panel_props()`;
- event logging reads output through `LinePlotPanelBinding::output_untracked(...)`;
- component-author props (`LinePlotPanelProps`) remain available inside `fret-plot` and advanced
  plot composition demos.

# Decision

Add `LinePlotPanelBinding::output_untracked(...)` rather than exposing the binding's internal output
model or forcing event handlers through layout/paint invalidation reads. Event-phase diagnostics need
a plain observation API, while render/layout code should continue using the tracked
`output_layout(...)` / `output_paint(...)` helpers.

# Verification

- `cargo nextest run -p fret-examples --test basic_plot_demos_surface plot_demo_uses_manual_harness_declarative_line_plot_panel --no-fail-fast`
- `cargo nextest run -p fret-plot line_plot_binding_creates_props_with_state_and_output_without_public_raw_handles line_plot_binding_reads_output_without_exposing_output_model_handle --no-fail-fast`

# Next

Do not mechanically convert every plot demo to `LinePlotPanelBinding`. The remaining raw props in
linked, overlay, image, histogram, and chart demos are either component-author surfaces or different
plot families that need their own binding contracts.
