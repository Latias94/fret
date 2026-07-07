---
type: Work Progress
title: Chart multi-axis linked binding boundary enters surface policy
tags:
  - fret
  - chart
  - multi-axis
  - surface-policy
  - linked-chart
timestamp: 2026-07-07T15:09:49Z
---

# Summary

Promoted the `chart_multi_axis_demo.rs` source-level linked chart contract into the global surface
policy gate. The linked multi-axis proof remains an advanced/manual harness surface, but it now has
a repo-level guard that keeps linked chart state, panel props, diagnostic output reads, and
diagnostic engine updates routed through `ChartCanvasLinkedGroupBinding` and
`ChartCanvasLinkedPanelBinding`.

The same rule blocks regressions back to retained chart widgets, `LinkedChartGroup`/`LinkedChartMember`
manual wiring, raw `Model<ChartEngine>` storage, direct `ChartCanvasPanelProps` wiring, and retained
split/widget node creation.

# Changed Files

- `tools/check_surface_policy.py`: names the chart multi-axis owner constant and adds
  `advanced-surface-chart-multi-axis-linked-binding-boundary`.
- `tools/test_check_surface_policy.py`: adds fixture coverage for legacy retained linked-chart
  wiring and an allowed linked binding-routed fixture.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_chart_multi_axis_retained_linked_wiring_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_chart_multi_axis_linked_binding_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples chart_multi_axis_demo_uses_declarative_canvas_panel_with_linked_inputs --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
