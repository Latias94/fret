---
type: Work Progress
title: Plot drag binding boundary enters surface policy
tags:
  - fret
  - plot
  - drag-demo
  - surface-policy
  - declarative-plot
timestamp: 2026-07-07T15:17:41Z
---

# Summary

Promoted the `drag_demo.rs` manual-harness plot contract into the global surface policy gate. The
demo still owns low-level runner and pointer-event plumbing, but plot panel construction, drag
output reads, and drag feedback state writes now have a repo-level guard that keeps them routed
through `LinePlotPanelBinding`.

The rule blocks regressions back to retained plot canvases, raw `Model<...>` plot/output handles,
manual `LinePlotPanelProps` state/output wiring, direct `PlotOutput` model allocation, and direct
`state.plot_state.update(...)` feedback writes.

# Changed Files

- `tools/check_surface_policy.py`: names the plot drag owner constant and adds
  `advanced-surface-plot-drag-declarative-binding-boundary`.
- `tools/test_check_surface_policy.py`: adds fixture coverage for legacy retained/raw plot drag
  wiring and an allowed binding-routed fixture.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_plot_drag_legacy_retained_authoring_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_plot_drag_declarative_binding_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples drag_demo_uses_manual_harness_declarative_line_plot_panel --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
