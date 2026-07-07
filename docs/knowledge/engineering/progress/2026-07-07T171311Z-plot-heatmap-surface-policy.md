---
type: Work Progress
title: Plot heatmap surface policy gate
timestamp: 2026-07-07T17:13:11Z
tags:
  - fret-examples
  - surface-policy
  - plot
  - heatmap
status: verified
---

# Summary

Added a dedicated source-policy boundary for `apps/fret-examples/src/heatmap_demo.rs`.
The demo remains an advanced manual runner because it owns the `FnDriver` and `UiTree` lifecycle,
but heatmap model authoring, style props, panel props, and query output reads must stay routed
through `HeatmapPlotPanelBinding`.

# Truth

- `heatmap_demo.rs` is no longer covered only by the generic manual chart owner.
- A retained/manual plot-state regression is rejected by the Python surface policy fixture.
- The current declarative heatmap binding authoring shape remains allowed.
- The Rust source proof and Python compact markers agree on the same public authoring surface.

# Artifacts

- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`
- `apps/fret-examples/tests/basic_plot_demos_surface.rs`

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_plot_heatmap_legacy_retained_authoring_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_plot_heatmap_declarative_binding_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples heatmap_demo_uses_manual_harness_declarative_heatmap_plot_panel --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Notes

- `cargo nextest` still reports the pre-existing `visual_map_track_at` dead-code warning in
  `ecosystem/fret-chart/src/visual_map_logic.rs`.
- Engineering wiki validation passed with existing migration warnings about missing `registry/`,
  large rollups, historical absolute paths, and historical progress/audit artifacts.
