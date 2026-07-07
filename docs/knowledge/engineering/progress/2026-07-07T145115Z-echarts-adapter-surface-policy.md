---
type: Work Progress
title: ECharts adapter binding boundary enters surface policy
tags:
  - fret
  - echarts
  - surface-policy
  - declarative-chart
timestamp: 2026-07-07T14:51:15Z
---

# Summary

Promoted the `echarts_demo.rs` source-level adapter smoke contract into the global surface policy
gate. The ECharts adapter comparison surface now has a repo-level guard that keeps chart titles on
the shared section chrome text role, mounts charts through `ChartCanvasPanelBinding` and
`chart_canvas_panel`, and blocks regressions back to raw chart model/props or raw kit text wiring.

# Changed Files

- `tools/check_surface_policy.py`: names the ECharts adapter owner constant and adds
  `comparison-surface-echarts-adapter-binding-boundary`.
- `tools/test_check_surface_policy.py`: adds fixture coverage for raw chart model/text wiring
  regressions and an allowed chart-binding fixture.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_echarts_adapter_raw_chart_and_text_wiring_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_echarts_adapter_binding_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples echarts_demo_chart_titles_use_section_chrome_role echarts_demo_uses_chart_binding_for_adapter_smoke --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
