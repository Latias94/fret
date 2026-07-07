---
type: Work Progress
title: Plot stress declarative binding boundary enters surface policy
tags:
  - fret
  - plot-stress
  - surface-policy
  - declarative-plot
timestamp: 2026-07-07T14:35:07Z
---

# Summary

Promoted the `plot_stress_demo.rs` source-level declarative plot contract into the global surface
policy gate. The plot stress harness remains an internal harness, but it now has a repo-level guard
that keeps panel construction on `LinePlotPanelBinding` plus declarative `line_plot_panel_in`, and
blocks regressions back to retained canvas/manual plot model wiring.

# Changed Files

- `tools/check_surface_policy.py`: names the plot stress owner constant and adds
  `internal-harness-plot-stress-declarative-binding-boundary`.
- `tools/test_check_surface_policy.py`: adds fixture coverage for retained/manual plot authoring
  regressions and an allowed declarative-binding fixture.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_plot_stress_legacy_retained_authoring_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_plot_stress_declarative_binding_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples plot_stress_demo_uses_manual_harness_declarative_line_plot_panel --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
