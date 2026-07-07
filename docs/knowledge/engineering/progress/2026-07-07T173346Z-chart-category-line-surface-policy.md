---
type: Work Progress
title: Chart category-line surface policy gate
timestamp: 2026-07-07T17:33:46Z
tags:
  - fret-examples
  - surface-policy
  - chart
  - category-line
status: verified
---

# Summary

Added a dedicated source-policy boundary for `apps/fret-examples/src/category_line_demo.rs`.
The demo remains an advanced manual runner because it owns the `FnDriver` and `UiTree` lifecycle,
but category axis setup, data zoom, initial data window, paint observation, panel props, and panel
wiring must stay routed through `ChartCanvasPanelBinding`.

# Truth

- `category_line_demo.rs` is no longer covered only by the generic manual chart owner.
- A retained/manual chart-canvas regression is rejected by the Python surface policy fixture.
- The current declarative chart-canvas binding authoring shape remains allowed.
- A Rust source proof now locks the production demo's category-line ChartCanvasPanelBinding shape.

# Artifacts

- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`
- `apps/fret-examples/tests/basic_plot_demos_surface.rs`

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_chart_category_line_legacy_retained_authoring_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_chart_category_line_declarative_binding_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples category_line_demo_uses_manual_harness_declarative_chart_canvas_panel_binding --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Notes

- `category_line_demo.rs` does not read chart output; this gate intentionally verifies category
  axis, data zoom, initial window, and panel wiring.
- `cargo nextest` still reports the pre-existing `visual_map_track_at` dead-code warning in
  `ecosystem/fret-chart/src/visual_map_logic.rs`.
- Engineering wiki validation passed with existing migration warnings about missing `registry/`,
  large rollups, historical absolute paths, and historical progress/audit artifacts.
