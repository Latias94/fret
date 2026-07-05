---
type: Work Progress
title: Plot declarative demo binding cleanup
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-plot-binding
tags: fret,ui-framework,public-surface,plot,raw-model
---

# Summary

`plot_declarative_demo` now uses `LinePlotPanelBinding` instead of storing a raw
`Model<LinePlotModel>` in the app view. This makes the default plot app example follow the existing
app-facing plot binding contract:

- the binding owns the plot model, interaction state, and output model;
- component authors can still use `LinePlotPanelProps` and raw models where needed;
- first-contact app code does not import `fret_runtime::Model`.

# Decisions

- Use the existing `LinePlotPanelBinding` rather than inventing a new plot state abstraction.
- Keep raw `LinePlotPanelProps` in advanced plot examples that demonstrate overlays, tags, linked
  output, or explicit state/output wiring.
- Treat this as the tracer bullet for broader plot/chart binding cleanup.

# Verification

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test basic_plot_demos_surface plot_declarative_demo_uses_default_declarative_line_plot_panel --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test app_import_surface examples_src_keeps_local_state_raw_bridges_out app_state_demos_use_app_local_state_imports --no-fail-fast`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`

# Follow-Up

- Add equivalent bindings for other plot families only when their demos need to be first-contact
  app examples.
- Chart and custom-effect demos still need separate binding contracts; do not mechanically wrap
  them until their state/output/control surfaces are designed.
