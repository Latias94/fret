---
type: Work Progress
title: Plot histogram source policy gate
timestamp: 2026-07-07T16:47:49Z
tags:
  - fret-examples
  - source-policy
  - plot-binding
  - histogram-plot
status: ready-for-commit
---

# Summary

The histogram plot demo now has an explicit source-policy owner, `examples-plot-histogram`,
instead of relying on the generic retained-chart manual demo classification.

# Outcome Truth

- `apps/fret-examples/src/histogram_demo.rs` may remain an advanced manual examples surface because
  it owns an `FnDriver`/`UiTree` runner.
- Histogram series creation, bin/gap settings, and query output reads must stay on
  `HistogramPlotPanelBinding`.
- The demo must not regress to retained `HistogramPlotCanvas`, raw `PlotState`/`PlotOutput`, or
  direct `HistogramPlotPanelProps::new(...)` authoring.

# Evidence

- `tools/check_surface_policy.py`: adds the histogram owner, required/forbidden compact markers,
  and a `CompactSourceBoundary` entry for
  `advanced-surface-plot-histogram-declarative-binding-boundary`.
- `tools/test_check_surface_policy.py`: adds positive and negative fixture coverage for
  declarative histogram plot authoring.
- `apps/fret-examples/tests/basic_plot_demos_surface.rs`: existing local example gate remains the
  Rust-side proof for the production demo.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_plot_histogram_legacy_retained_authoring_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_plot_histogram_declarative_binding_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples histogram_demo_uses_manual_harness_declarative_histogram_plot_panel --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Notes

The `cargo nextest` run still prints the existing `visual_map_track_at` dead-code warning in
`ecosystem/fret-chart/src/visual_map_logic.rs`; this slice does not touch that crate.
