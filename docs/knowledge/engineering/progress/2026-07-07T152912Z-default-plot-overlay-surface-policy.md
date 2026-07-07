---
type: Work Progress
title: Default plot overlay demos enter surface policy
tags:
  - fret
  - plot
  - default-authoring
  - surface-policy
  - documentation-ordering
timestamp: 2026-07-07T15:29:12Z
---

# Summary

Promoted the `tags_demo.rs` and `plot_image_demo.rs` default plot overlay contracts into the
global surface policy gate. These examples are copyable default app-facing `View` demos, while their
native/web launch glue lives in small internal driver modules.

The new policy keeps the default source files on `fret::app::prelude::*`,
`LinePlotPanelBinding`, and `line_plot_panel_in(...)`, including overlay state setup for tags/text
and dynamic image overlay state updates for plot images. It also keeps the companion drivers on the
shared default-view launch helpers instead of reintroducing hand-written `FnDriver`/runtime wiring.

# Changed Files

- `tools/check_surface_policy.py`: adds default plot overlay source and driver boundary scanners.
- `tools/test_check_surface_policy.py`: adds fixture coverage for retained/raw plot overlay
  regressions, hand-written driver regressions, and the allowed default `View` plus shared driver
  shape.

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_default_plot_overlay_retained_authoring_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_default_plot_overlay_binding_and_driver_surfaces_are_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples -E 'test(tags_demo_uses_default_declarative_line_plot_panel) | test(plot_image_demo_uses_default_declarative_line_plot_panel) | test(tags_demo_driver_owns_default_view_launch_wiring) | test(plot_image_demo_driver_owns_default_view_launch_wiring) | test(docs_index_separates_default_app_plot_demo_from_manual_harnesses)' --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
