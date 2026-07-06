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

The raw model handles are now centralized behind `PlotStressModelOwner`:

- `plot_model()` hands the render path the model handle needed by `LinePlotPanelProps`;
- `animate_enabled(...)` and `toggle_animate(...)` own the animation toggle model;
- `shift_plot_bounds_for_animation(...)` owns the stress-only plot-model mutation.

# Decision

Do not migrate `plot_stress_demo` to `LinePlotPanelBinding`. The binding surface owns normal
interaction state/output, while this stress harness intentionally measures driver-owned model
mutation and renderer perf. The correct cleanup is a named local owner, not hiding the stress
semantics behind cookbook APIs.

# Verification

- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test basic_plot_demos_surface plot_stress_demo_uses_manual_harness_declarative_line_plot_panel --no-fail-fast`
- `python3 tools/examples_source_tree_policy/gate.py`

# Next

Continue with true advanced plot contract design for overlay and linked-panel demos:
`drag_demo`, `inf_lines_demo`, `tags_demo`, `plot_image_demo`, and `linked_cursor_demo`.
