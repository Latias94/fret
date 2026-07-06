---
type: "Work Progress"
title: "Gizmo3D theme keyboard binding"
description: "Work Progress for routing Gizmo3D theme preset model coordination through binding methods."
timestamp: 2026-07-06T23:17:29Z
tags: ["fret", "gizmo3d", "examples", "public-surface", "keyboard", "theme", "binding"]
git_branch: "refactor/gizmo3d-theme-keyboard-owner"
verified_by: "cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_theme_keyboard_mutations_through_binding --no-fail-fast"
---

# Summary

`gizmo3d_demo.rs` now routes KeyY theme preset model coordination through
`Gizmo3dDemoModelBinding` request/commit methods instead of opening direct `state.demo.update(...)`
closures inside the event branch.

# Details

- Added `DemoThemePresetRequest` as the narrow handoff from the binding-owned model state to the
  event branch.
- Added `next_theme_preset_request(...)` to compute the next preset only when the gizmo model is
  not busy.
- Added `apply_theme_preset(...)` to commit the preset index and re-apply viewport gizmo theme
  tokens after the global Theme has been updated.
- Kept file IO, `ThemeConfig` parsing, and global `Theme` mutation in the KeyY event branch because
  those are orchestration concerns outside the demo model owner.
- Extended `gizmo3d_demo_surface.rs` to require the request/commit calls and reject the old direct
  event-branch model writes.

# Verification

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_theme_keyboard_mutations_through_binding --no-fail-fast`
  failed because the theme keyboard binding methods were not used.
- `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_theme_keyboard_mutations_through_binding --no-fail-fast`

# Next Action

Continue with viewport-input routing and render-HUD updates as separate slices. Viewport input
touches undo records and tool arbitration; render-HUD updates happen in the paint path.

# Citations

- [gizmo3d_demo.rs](../../../../apps/fret-examples/src/gizmo3d_demo.rs)
- [gizmo3d_demo_surface.rs](../../../../apps/fret-examples/tests/gizmo3d_demo_surface.rs)
