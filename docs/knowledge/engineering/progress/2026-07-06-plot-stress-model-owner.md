---
type: Work Progress
title: Plot stress model owner cleanup
timestamp: 2026-07-06T01:22:00Z
git_branch: main
tags: fret,ui-framework,public-surface,plot,stress,raw-model,owner
---

# Summary

`plot_stress_demo` remains a maintainer/perf harness rather than a default app-facing plot example.
It deliberately mutates `LinePlotModel::data_bounds` from the driver loop to force path rebuilds.

The original cleanup centralized raw model handles behind `PlotStressModelOwner`:

- `plot_model()` hands the render path the model handle needed by `LinePlotPanelProps`;
- `animate_enabled(...)` and `toggle_animate(...)` own the animation toggle model;
- `shift_plot_bounds_for_animation(...)` owns the stress-only plot-model mutation.

A later cleanup moved the stress plot itself behind `LinePlotPanelBinding` after the binding gained
`read_model_untracked(...)` and `update_model(...)`. `PlotStressModelOwner` now keeps the animation
toggle plus the binding-backed plot mutation, so the demo no longer stores `Model<LinePlotModel>` or
builds `LinePlotPanelProps` manually.

# Decision

The earlier decision not to migrate to `LinePlotPanelBinding` was valid while the binding could not
express driver-owned plot model mutation. With `LinePlotPanelBinding::update_model(...)`, the stress
semantics stay explicit without exposing the raw model handle to the demo.

# Verification

- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-plot line_plot_binding_updates_model_without_exposing_model_handle --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_plot_demos_surface plot_stress_demo_uses_manual_harness_declarative_line_plot_panel --no-fail-fast`
- `python3 tools/examples_source_tree_policy/gate.py`

# Next

The advanced overlay and linked-panel examples later moved onto the same binding family through
initial state, state mutation, output reads, and linked-member APIs. Continue looking for raw model
seams where app examples still expose framework choreography instead of a component-specific owner.
