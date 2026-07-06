---
type: Work Progress
title: Stairs plot binding cleanup
timestamp: 2026-07-06T01:08:00Z
git_branch: main
tags: fret,ui-framework,public-surface,plot,line,binding,raw-model
---

# Summary

`stairs_demo` now stores a `LinePlotPanelBinding` instead of separate raw
`Model<LinePlotModel>`, `Model<PlotState>`, and `Model<PlotOutput>` handles.

The demo still teaches `StepMode::Post`, but step mode is just a declarative props option layered
on top of `LinePlotPanelBinding::panel_props()`. It does not need a separate advanced state/output
contract.

# Decision

Keep simple line-plot presentation options, such as `step_mode(...)`, on the binding-derived props
path. Reserve raw `LinePlotPanelProps::new(Model<_>)` in demos for cases that truly need externally
owned plot state, overlays, or linked panels.

# Verification

- `cargo nextest run -p fret-examples --test basic_plot_demos_surface stairs_demo_uses_manual_harness_declarative_line_plot_panel_with_step_mode --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`

# Next

The line-plot examples that were raw at this point later moved onto explicit
`LinePlotPanelBinding` contracts:
initial overlay state, event-time output reads, state mutation, linked members, and controlled model
mutation. Keep raw line plot props for component-author tests and true custom composition rather
than default example authoring.
