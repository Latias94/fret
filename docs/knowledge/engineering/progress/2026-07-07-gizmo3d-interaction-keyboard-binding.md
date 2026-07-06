---
type: "Work Progress"
title: "Gizmo3D interaction keyboard binding"
description: "Work Progress for routing Gizmo3D op-mask, orientation, selection, and camera keyboard mutations through binding methods."
timestamp: 2026-07-06T23:06:49Z
tags: ["fret", "gizmo3d", "examples", "public-surface", "keyboard", "binding"]
git_branch: "refactor/gizmo3d-selection-keyboard-owner"
verified_by: "cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_interaction_keyboard_mutations_through_binding --no-fail-fast"
---

# Summary

`gizmo3d_demo.rs` now routes the remaining pure keyboard interaction mutation cluster through named
`Gizmo3dDemoModelBinding` methods instead of opening direct `state.demo.update(...)` closures inside
event branches.

# Details

- Added `cycle_op_mask_preset(...)` for bracket-driven op-mask preset cycling.
- Added `toggle_gizmo_orientation(...)` and `toggle_gizmo_pivot_mode(...)`.
- Added `cycle_active_target(...)` for next/previous active selection shortcuts.
- Added `frame_targets(...)` for keyboard camera framing.
- Added `apply_select_all_shortcut(...)` and `apply_target_selection_shortcut(...)` for selection
  shortcuts.
- Updated the BracketLeft, BracketRight, KeyL, KeyP, KeyN, KeyB, KeyF, Ctrl/Cmd+A, and Digit1-3
  event branches to call binding methods while preserving the existing redraw behavior.
- Extended `gizmo3d_demo_surface.rs` to require those binding calls and reject the old direct event
  branch model writes.

# Verification

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_interaction_keyboard_mutations_through_binding --no-fail-fast`
  failed because the interaction keyboard binding methods were not used.
- `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_interaction_keyboard_mutations_through_binding --no-fail-fast`

# Next Action

Split the remaining nontrivial Gizmo3D model writes into separate slices: theme preset switching
because it crosses file IO and global theme mutation, viewport-input routing because it produces
undo records and tool arbitration, and render-HUD updates because they happen in the paint path.

# Citations

- [gizmo3d_demo.rs](../../../../apps/fret-examples/src/gizmo3d_demo.rs)
- [gizmo3d_demo_surface.rs](../../../../apps/fret-examples/tests/gizmo3d_demo_surface.rs)
