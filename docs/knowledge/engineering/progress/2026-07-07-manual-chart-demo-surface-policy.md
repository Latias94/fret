---
type: "Work Progress"
title: "Manual chart demo surface policy"
description: "Work Progress for classifying manual retained chart demos as advanced public examples."
timestamp: 2026-07-07T01:54:40Z
tags: ["fret", "charts", "examples", "advanced-surface", "source-policy"]
git_branch: "refactor/chart-plot-demo-surface-policy"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

The manual retained chart demo cluster is now included in public example scanning and classified as
advanced/manual. These demos all own the same direct `FnDriver`/`UiTree` chart lifecycle seams, so
they are explicitly quarantined as manual examples instead of being treated as default app-authoring
surfaces.

# Details

- Added scan roots and advanced/manual classifications for:
  `area_demo.rs`, `bars_demo.rs`, `candlestick_demo.rs`, `category_line_demo.rs`,
  `chart_demo.rs`, `error_bars_demo.rs`, `grouped_bars_demo.rs`, `heatmap_demo.rs`,
  `histogram2d_demo.rs`, `histogram_demo.rs`, `horizontal_bars_demo.rs`, `inf_lines_demo.rs`,
  `linked_cursor_demo.rs`, `plot3d_demo.rs`, `shaded_demo.rs`, `stacked_bars_demo.rs`,
  `stairs_demo.rs`, and `stems_demo.rs`.
- Introduced `MANUAL_CHART_DEMO_FILENAMES` and a shared `_fret_examples_manual_chart_surface(...)`
  helper to keep the classification precise without adding a directory-level scan root.
- The allowed raw seams are intentionally exact for this cluster:
  `fret_app`, `fret_core`, `fret_launch`, `fret_runtime`, `fret_ui`, `FnDriver`, and `UiTree`.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
  failed because the manual chart demo paths were not in `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Continue classifying remaining raw/advanced example gaps. The next likely clusters are
custom-effect v1/v2/v3 native proofs, streaming import demos, and small manual smoke/probe demos.

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
