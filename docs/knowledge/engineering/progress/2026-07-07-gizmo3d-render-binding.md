---
type: "Work Progress"
title: "Gizmo3D render binding"
description: "Work Progress for routing Gizmo3D render-path cursor-scale synchronization through a binding method."
timestamp: 2026-07-06T23:42:46Z
tags: ["fret", "gizmo3d", "examples", "public-surface", "render", "binding"]
git_branch: "refactor/gizmo3d-render-binding"
verified_by: "cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_render_cursor_scale_through_binding --no-fail-fast"
---

# Summary

`gizmo3d_demo.rs` now routes render-path cursor-scale synchronization through
`Gizmo3dDemoModelBinding` instead of opening a direct `state.demo.update(...)` closure in the render
function.

# Details

- Added `sync_cursor_scale_from_viewport(...)` to keep viewport cursor scale application behind the
  demo model binding.
- Replaced the render-path `state.demo.update(...)` call with that binding method.
- Extended `gizmo3d_demo_surface.rs` to require the binding call and reject the old direct
  render-path model write.

# Verification

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_render_cursor_scale_through_binding --no-fail-fast`
  failed because render cursor-scale sync was not routed through the binding method.
- `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_render_cursor_scale_through_binding --no-fail-fast`

# Next Action

Continue with `viewport_input(...)` as the remaining large Gizmo3D direct update boundary. It should
stay separate because it performs tool arbitration and produces undo records.

# Citations

- [gizmo3d_demo.rs](../../../../apps/fret-examples/src/gizmo3d_demo.rs)
- [gizmo3d_demo_surface.rs](../../../../apps/fret-examples/tests/gizmo3d_demo_surface.rs)
