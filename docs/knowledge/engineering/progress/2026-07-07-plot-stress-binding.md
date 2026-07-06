---
type: "Work Progress"
title: "Plot stress binding cleanup"
description: "Work Progress for the plot stress binding cleanup."
timestamp: 2026-07-07T02:20:00Z
tags: ["fret", "plot", "examples", "stress", "public-surface", "raw-model", "binding"]
git_branch: "refactor/plot-stress-binding"
verified_by: "cargo nextest run -p fret-examples --test basic_plot_demos_surface plot_stress_demo_uses_manual_harness_declarative_line_plot_panel --no-fail-fast"
---

# Summary

`plot_stress_demo.rs` now keeps its stress line plot behind `LinePlotPanelBinding` instead of
storing `Model<LinePlotModel>` and manually building `LinePlotPanelProps`.

# Details

- Added `read_model_untracked(...)` and `update_model(...)` to plot panel bindings so advanced
  examples can inspect or mutate a controlled plot model without exposing the raw model handle.
- Migrated `plot_stress_demo.rs` so `PlotStressModelOwner` stores `LinePlotPanelBinding`, renders
  with `panel_props()`, and shifts stress bounds through `LinePlotPanelBinding::update_model(...)`.
- Strengthened `basic_plot_demos_surface` so the stress demo must keep the binding shape and must
  not regress to raw `Model<LinePlotModel>`, direct model insertion, or manual
  `LinePlotPanelProps::new(plot.clone())` wiring.

# Verification

- `cargo fmt --all --check`
- `cargo check -p fret-plot`
- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-plot line_plot_binding_updates_model_without_exposing_model_handle --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_plot_demos_surface plot_stress_demo_uses_manual_harness_declarative_line_plot_panel --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

Keep raw plot props in component tests and truly custom composition paths. For examples that only
need diagnostics, output, state, or controlled model mutation, prefer extending the binding surface
before exposing raw `Model<T>`.

# Citations

- [binding.rs](../../../../ecosystem/fret-plot/src/binding.rs)
- [plot_stress_demo.rs](../../../../apps/fret-examples/src/plot_stress_demo.rs)
- [basic_plot_demos_surface.rs](../../../../apps/fret-examples/tests/basic_plot_demos_surface.rs)
