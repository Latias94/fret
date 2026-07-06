---
type: "Work Progress"
title: "Gizmo3D Plot3D binding"
description: "Work Progress for migrating the Gizmo3D demo's Plot3D panel state to Plot3dPanelBinding."
timestamp: 2026-07-06T22:17:03Z
tags: ["fret", "gizmo3d", "plot3d", "examples", "public-surface", "raw-model"]
git_branch: "refactor/gizmo3d-plot-binding"
verified_by: "cargo nextest run -p fret-examples --test gizmo3d_demo_surface --no-fail-fast"
---

# Summary

`gizmo3d_demo.rs` now uses `Plot3dPanelBinding` for its embedded Plot3D panel instead of storing a
raw `fret_runtime::Model<Plot3dModel>` in the window state.

# Details

- Replaced the `plot` window-state field with `Plot3dPanelBinding`.
- Replaced Plot3D model allocation with `Plot3dPanelBinding::new(...)`.
- Replaced render-target size reads and target sync writes with `viewport_untracked(...)` and
  `sync_viewport_target(...)`.
- Replaced the render path's manual `Plot3dPanelProps::new(state.plot.clone())` call with
  `state.plot.panel_props()`.
- Replaced the view-gizmo label mapping read with the binding's viewport snapshot.
- Added `gizmo3d_demo_surface.rs` to prevent this demo from reintroducing raw Plot3D model handles.

# Verification

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_uses_app_facing_plot3d_binding --no-fail-fast`
  failed because `Plot3dPanelBinding` was not imported or used.
- `cargo nextest run -p fret-examples --test gizmo3d_demo_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --test plot3d_demo_surface --test gizmo3d_demo_surface --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`
- `cargo fmt --all --check`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

Continue with the larger `Gizmo3dDemoModel` mutation surface. That pass should introduce a
demo-local model owner/binding instead of mechanically replacing the editor/gizmo shared state with
view-local state.

# Citations

- [gizmo3d_demo.rs](../../../../apps/fret-examples/src/gizmo3d_demo.rs)
- [gizmo3d_demo_surface.rs](../../../../apps/fret-examples/tests/gizmo3d_demo_surface.rs)
