---
type: Work Progress
title: Plot stairs source policy gate
timestamp: 2026-07-07T16:26:36Z
tags:
  - fret-examples
  - source-policy
  - plot-binding
  - stairs-plot
status: ready-for-commit
---

# Summary

The stairs plot demo now has an explicit source-policy owner, `examples-plot-stairs`, instead of
relying on the generic retained-chart manual demo classification.

# Outcome Truth

- `apps/fret-examples/src/stairs_demo.rs` may remain an advanced manual examples surface because it
  owns an `FnDriver`/`UiTree` runner.
- Line plot model creation, `StepMode::Post` panel authoring, and query output reads must stay on
  `LinePlotPanelBinding`.
- The demo must not regress to retained `StairsPlotCanvas`, raw `PlotState`/`PlotOutput`, or direct
  `LinePlotPanelProps::new(...)` state/output wiring.

# Evidence

- `tools/check_surface_policy.py`: adds the stairs owner, required/forbidden compact markers, and
  a `CompactSourceBoundary` entry for
  `advanced-surface-plot-stairs-declarative-binding-boundary`.
- `tools/test_check_surface_policy.py`: adds positive and negative fixture coverage for declarative
  step-mode line plot authoring.
- `apps/fret-examples/tests/basic_plot_demos_surface.rs`: existing local example gate remains the
  Rust-side proof for the production demo.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_plot_stairs_legacy_retained_authoring_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_plot_stairs_declarative_binding_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples stairs_demo_uses_manual_harness_declarative_line_plot_panel_with_step_mode --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Notes

The `cargo nextest` run still prints the existing `visual_map_track_at` dead-code warning in
`ecosystem/fret-chart/src/visual_map_logic.rs`; this slice does not touch that crate.
