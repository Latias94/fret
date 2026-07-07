---
type: Work Progress
title: Plot error bars source policy gate
timestamp: 2026-07-07T16:41:20Z
tags:
  - fret-examples
  - source-policy
  - plot-binding
  - error-bars-plot
status: ready-for-commit
---

# Summary

The error-bars plot demo now has an explicit source-policy owner, `examples-plot-error-bars`,
instead of relying on the generic retained-chart manual demo classification.

# Outcome Truth

- `apps/fret-examples/src/error_bars_demo.rs` may remain an advanced manual examples surface
  because it owns an `FnDriver`/`UiTree` runner.
- Error-bar model creation, x/y error series wiring, and query output reads must stay on
  `ErrorBarsPlotPanelBinding`.
- The demo must not regress to retained `ErrorBarsPlotCanvas`, raw `PlotState`/`PlotOutput`, or
  direct `ErrorBarsPlotPanelProps::new(...)` state/output wiring.

# Evidence

- `tools/check_surface_policy.py`: adds the error-bars owner, required/forbidden compact markers,
  and a `CompactSourceBoundary` entry for
  `advanced-surface-plot-error-bars-declarative-binding-boundary`.
- `tools/test_check_surface_policy.py`: adds positive and negative fixture coverage for
  declarative error-bars plot authoring.
- `apps/fret-examples/tests/basic_plot_demos_surface.rs`: existing local example gate remains the
  Rust-side proof for the production demo.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_plot_error_bars_legacy_retained_authoring_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_plot_error_bars_declarative_binding_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples error_bars_demo_uses_manual_harness_declarative_error_bars_plot_panel --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Notes

The `cargo nextest` run still prints the existing `visual_map_track_at` dead-code warning in
`ecosystem/fret-chart/src/visual_map_logic.rs`; this slice does not touch that crate.
