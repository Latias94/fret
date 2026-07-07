---
type: "Work Progress"
title: "Gizmo3D viewport input binding"
description: "Work Progress for routing Gizmo3D viewport input model handling through a binding method."
timestamp: 2026-07-06T23:58:57Z
tags: ["fret", "gizmo3d", "examples", "public-surface", "viewport-input", "binding"]
git_branch: "refactor/gizmo3d-viewport-input-binding"
verified_by: "cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_viewport_input_through_binding --no-fail-fast"
---

# Summary

`gizmo3d_demo.rs` now routes viewport input model handling through
`Gizmo3dDemoModelBinding::handle_viewport_input(...)` instead of opening a direct
`model.update(...)` closure in the free `viewport_input(...)` function.

# Details

- Added `handle_viewport_input(...)` to the demo model binding.
- Moved the existing viewport input model logic into that binding method: target validation,
  cursor-scale sync, camera orbit/pan/zoom, viewport-tool arbitration, transform-gizmo updates,
  HUD state refresh, and pending undo-record construction.
- Kept undo service recording and redraw scheduling in `viewport_input(...)`, preserving the
  orchestration boundary outside the model owner.
- Extended `gizmo3d_demo_surface.rs` to require the binding call and reject the old direct
  viewport-input model update in the free function.

# Verification

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_viewport_input_through_binding --no-fail-fast`
  failed because viewport input was not routed through the binding method.
- `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_viewport_input_through_binding --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`

# Next Action

Continue with `record_engine_frame(...)` frame animation and draw snapshot updates. They remain as
the last app-side direct `state.demo.update(...)` callers in `gizmo3d_demo.rs`.

# Citations

- [gizmo3d_demo.rs](../../../../apps/fret-examples/src/gizmo3d_demo.rs)
- [gizmo3d_demo_surface.rs](../../../../apps/fret-examples/tests/gizmo3d_demo_surface.rs)
