---
type: Work Progress
title: ECharts multi-grid binding boundary enters surface policy
tags:
  - fret
  - echarts
  - multi-grid
  - surface-policy
  - declarative-chart
timestamp: 2026-07-07T14:59:01Z
---

# Summary

Promoted the `echarts_multi_grid_demo.rs` source-level multi-grid chart contract into the global
surface policy gate. The ECharts multi-grid proof remains an advanced/manual surface, but it now has
a repo-level guard that keeps per-grid panels and overlay-only panel construction on
`ChartCanvasMultiGridBinding` plus `chart_canvas_panel`, and blocks regressions back to retained
multi-grid helpers or raw engine model wiring.

# Changed Files

- `tools/check_surface_policy.py`: names the ECharts multi-grid owner constant and adds
  `advanced-surface-echarts-multi-grid-binding-boundary`.
- `tools/test_check_surface_policy.py`: adds fixture coverage for retained multi-grid helper/raw
  engine regressions and an allowed binding-routed fixture.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_echarts_multi_grid_retained_helpers_are_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_echarts_multi_grid_binding_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples echarts_multi_grid_demo_uses_declarative_grid_panels_and_overlay --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
