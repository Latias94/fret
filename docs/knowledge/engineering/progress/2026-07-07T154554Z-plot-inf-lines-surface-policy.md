---
type: Work Progress
title: Inf-lines plot binding boundary enters surface policy
tags:
  - fret
  - plot
  - inf-lines
  - surface-policy
  - declarative-plot
timestamp: 2026-07-07T15:45:54Z
---

# Summary

Promoted the `inf_lines_demo.rs` manual plot overlay contract into the global surface policy gate.
The demo still owns manual `FnDriver`/`UiTree` runner seams, but infinite-line overlay setup, query
output reads, multi-axis label configuration, and panel construction now have a repo-level guard
that keeps them routed through `LinePlotPanelBinding` and `line_plot_panel_in(...)`.

The rule also corrects the source-policy owner wording for this surface from the generic retained
chart bucket to an explicit plot inf-lines owner, and blocks regressions back to retained plot
canvases, raw `Model<...>` plot/output handles, manual `LinePlotPanelProps` state/output wiring, or
direct `PlotOutput` model allocation.

# Changed Files

- `tools/check_surface_policy.py`: adds `examples-plot-inf-lines`, required/forbidden compact
  markers, and `advanced-surface-plot-inf-lines-declarative-binding-boundary`.
- `tools/test_check_surface_policy.py`: adds fixture coverage for legacy retained/raw inf-lines
  plot wiring and the allowed binding-routed shape.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_plot_inf_lines_legacy_retained_authoring_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_plot_inf_lines_declarative_binding_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples inf_lines_demo_uses_manual_harness_declarative_line_plot_panel --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
