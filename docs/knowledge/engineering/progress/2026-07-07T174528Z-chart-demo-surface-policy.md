---
type: Work Progress
title: Chart demo surface policy gate
timestamp: 2026-07-07T17:45:28Z
tags:
  - fret-examples
  - surface-policy
  - chart
  - chart-demo
status: verified
---

# Summary

Added a dedicated source-policy boundary for `apps/fret-examples/src/chart_demo.rs`.
The demo remains an advanced manual runner because it owns the `FnDriver` and `UiTree` lifecycle,
but time-axis setup, left/right axis authoring, stacked area series, right-axis line series, dataset
insertion, paint observation, panel props, and panel wiring must stay routed through
`ChartCanvasPanelBinding`.

# Truth

- `chart_demo.rs` is no longer covered only by the generic manual chart owner.
- A retained/manual chart-canvas regression is rejected by the Python surface policy fixture.
- The current declarative chart-canvas binding authoring shape remains allowed.
- A Rust source proof now locks the production demo's multi-axis ChartCanvasPanelBinding shape.

# Artifacts

- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`
- `apps/fret-examples/tests/basic_plot_demos_surface.rs`

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_chart_demo_legacy_retained_authoring_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_chart_demo_declarative_binding_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples chart_demo_uses_manual_harness_declarative_chart_canvas_panel_binding --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Notes

- This gate intentionally verifies the canonical mixed chart shape: time axis, stacked area series,
  a right-axis line series, dataset insertion, and declarative panel wiring.
- `cargo nextest` still reports the pre-existing `visual_map_track_at` dead-code warning in
  `ecosystem/fret-chart/src/visual_map_logic.rs`.
- Engineering wiki validation passed with existing migration warnings about missing `registry/`,
  large rollups, historical absolute paths, and historical progress/audit artifacts.
