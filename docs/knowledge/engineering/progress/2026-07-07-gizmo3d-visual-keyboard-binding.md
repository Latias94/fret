---
type: "Work Progress"
title: "Gizmo3D visual keyboard binding"
description: "Work Progress for routing Gizmo3D visual preset and size-policy keyboard mutations through binding methods."
timestamp: 2026-07-06T22:52:27Z
tags: ["fret", "gizmo3d", "examples", "public-surface", "keyboard", "binding"]
git_branch: "refactor/gizmo3d-visual-keyboard-owner"
verified_by: "cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_visual_keyboard_mutations_through_binding --no-fail-fast"
---

# Summary

`gizmo3d_demo.rs` now routes the visual preset and size-policy keyboard mutation cluster through
named `Gizmo3dDemoModelBinding` methods instead of opening direct `state.demo.update(...)` closures
inside the event branches.

# Details

- Added `cycle_visual_preset(...)` for transform-gizmo and view-gizmo visual preset cycling.
- Added `cycle_size_policy(...)` and `adjust_size_policy_fraction(...)` for size-policy mode and
  fraction changes.
- Added `adjust_gizmo_size_px(...)` and `adjust_gizmo_stroke_px(...)` for keyboard-driven visual
  scale and stroke changes.
- Updated the KeyG, KeyV, Semicolon, Quote, Minus, Equal, Comma, and Period event branches to call
  binding methods while preserving the existing redraw behavior.
- Extended `gizmo3d_demo_surface.rs` to require those binding calls and reject the old direct event
  branch model writes.

# Verification

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_visual_keyboard_mutations_through_binding --no-fail-fast`
  failed because the visual keyboard binding methods were not used.
- `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_visual_keyboard_mutations_through_binding --no-fail-fast`

# Next Action

Continue with the selection/camera keyboard shortcuts, then viewport-input routing and render-HUD
updates as separate slices because they touch undo records, camera state, tool arbitration, and
paint-time overlay state.

# Citations

- [gizmo3d_demo.rs](../../../../apps/fret-examples/src/gizmo3d_demo.rs)
- [gizmo3d_demo_surface.rs](../../../../apps/fret-examples/tests/gizmo3d_demo_surface.rs)
