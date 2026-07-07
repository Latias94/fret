---
type: Work Progress
title: Plot declarative demo enters default surface policy
tags:
  - fret
  - plot
  - default-authoring
  - surface-policy
  - documentation-ordering
timestamp: 2026-07-07T15:37:07Z
---

# Summary

Promoted `plot_declarative_demo.rs` into the global default authoring surface policy. The docs index
already presents this demo as the default `FretApp + View` plot example, and the source policy now
matches that ordering: the file is scanned as a default-clean public example and must stay on
`fret::app::prelude::*`, `LinePlotPanelBinding`, and `line_plot_panel_in(...)`.

The same gate rejects regressions back to retained plot canvases, raw plot model handles,
`LinePlotPanelProps` manual wiring, or driver-level `FnDriver` vocabulary in this default source.

# Changed Files

- `tools/check_surface_policy.py`: adds `plot_declarative_demo.rs` to default authoring surfaces,
  public example scan roots, and the default plot binding boundary map.
- `tools/test_check_surface_policy.py`: extends default plot policy fixtures and scan-root
  assertions to include `plot_declarative_demo.rs`.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_default_plot_overlay_binding_and_driver_surfaces_are_allowed tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples plot_declarative_demo_uses_default_declarative_line_plot_panel --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
