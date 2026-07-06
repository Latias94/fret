---
type: "Work Progress"
title: "Gizmo3D undo binding"
description: "Work Progress for routing Gizmo3D undo/redo model replay through binding methods."
timestamp: 2026-07-06T23:30:15Z
tags: ["fret", "gizmo3d", "examples", "public-surface", "undo", "binding"]
git_branch: "refactor/gizmo3d-undo-model-owner"
verified_by: "cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_undo_redo_mutations_through_binding --no-fail-fast"
---

# Summary

`gizmo3d_demo.rs` now routes undo/redo model replay through `Gizmo3dDemoModelBinding` methods
instead of opening direct `state.demo.update(...)` closures inside the driver.

# Details

- Reused `cancel_active_or_in_progress(...)` for the undo/redo preflight cancel step.
- Added `apply_target_transforms(...)` for transform undo/redo replay.
- Added `apply_custom_scalar_values(...)` for custom scalar undo/redo replay.
- Updated `handle_undo_redo_shortcut(...)` so the undo service callbacks call binding methods
  instead of mutating `targets` or `custom_scalar_values` directly.
- Extended `gizmo3d_demo_surface.rs` to require those binding calls and reject the old direct
  driver model writes.

# Verification

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_undo_redo_mutations_through_binding --no-fail-fast`
  failed because undo/redo replay was not routed through binding methods.
- `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_undo_redo_mutations_through_binding --no-fail-fast`

# Next Action

Continue with viewport-input routing and render-HUD updates as separate slices. Viewport input
touches undo record creation and tool arbitration; render-HUD updates happen in the paint path.

# Citations

- [gizmo3d_demo.rs](../../../../apps/fret-examples/src/gizmo3d_demo.rs)
- [gizmo3d_demo_surface.rs](../../../../apps/fret-examples/tests/gizmo3d_demo_surface.rs)
