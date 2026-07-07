---
type: Work Progress
title: Chart stress declarative binding boundary enters surface policy
tags:
  - fret
  - chart-stress
  - surface-policy
  - declarative-chart
timestamp: 2026-07-07T14:44:46Z
---

# Summary

Promoted the `chart_stress_demo.rs` source-level declarative chart contract into the global surface
policy gate. The chart stress harness remains an internal harness, but it now has a repo-level guard
that keeps chart panel construction on `ChartCanvasPanelBinding` plus declarative
`chart_canvas_panel`, preserves engine-paint observation and stats reporting, and blocks regressions
back to retained canvas/manual engine model wiring.

# Changed Files

- `tools/check_surface_policy.py`: names the chart stress owner constant and adds
  `internal-harness-chart-stress-declarative-binding-boundary`.
- `tools/test_check_surface_policy.py`: adds fixture coverage for retained/manual chart canvas
  regressions and an allowed declarative-binding fixture.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_chart_stress_legacy_retained_authoring_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_chart_stress_declarative_binding_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples chart_stress_demo_uses_declarative_canvas_panel --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
