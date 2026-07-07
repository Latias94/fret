---
type: Work Progress
title: Plot3D surface policy gate
timestamp: 2026-07-07T17:52:52Z
tags:
  - fret-examples
  - surface-policy
  - plot3d
  - viewport
status: verified
---

# Summary

Added a dedicated source-policy boundary for `apps/fret-examples/src/plot3d_demo.rs`.
The demo remains an advanced manual runner because it owns the `FnDriver`, `UiTree`, render-target
allocation, and engine-frame command-buffer hook, but viewport model state, render-target sync,
panel props, and panel wiring must stay routed through `Plot3dPanelBinding`.

# Truth

- `plot3d_demo.rs` is no longer covered only by the generic manual chart owner.
- Raw `Model<Plot3dModel>` handles, direct `Plot3dPanelProps::new(...)`, `plot3d_panel_with_model(...)`,
  and `Plot3dPanelBinding::from_model(...)` regressions are rejected by the Python surface policy fixture.
- The current app-facing `Plot3dPanelBinding` authoring shape remains allowed.
- The existing Rust source proof now also locks render-target state, target sync, engine-frame output,
  and declarative panel wiring anchors.

# Artifacts

- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`
- `apps/fret-examples/tests/plot3d_demo_surface.rs`

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_plot3d_raw_model_authoring_is_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_plot3d_binding_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-examples plot3d_demo_uses_app_facing_plot3d_binding --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Notes

- This gate intentionally allows the demo-owned WGPU render hook; the policy boundary is the
  app-facing Plot3D panel model and panel props surface.
- `cargo nextest` still reports the pre-existing `visual_map_track_at` dead-code warning in
  `ecosystem/fret-chart/src/visual_map_logic.rs`.
- Engineering wiki validation passed with existing migration warnings about missing `registry/`,
  large rollups, historical absolute paths, and historical progress/audit artifacts.
