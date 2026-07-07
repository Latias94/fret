---
type: "Work Progress"
title: "Gizmo3D frame render binding"
description: "Work Progress for routing Gizmo3D frame animation and render snapshot model access through binding methods."
timestamp: 2026-07-07T00:13:08Z
tags: ["fret", "gizmo3d", "examples", "public-surface", "render", "binding"]
git_branch: "refactor/gizmo3d-frame-render-binding"
verified_by: "cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_frame_render_mutations_through_binding --no-fail-fast"
---

# Summary

`gizmo3d_demo.rs` now routes frame animation and render snapshot model access through
`Gizmo3dDemoModelBinding` methods instead of opening direct `state.demo.update(...)` closures in
`record_engine_frame(...)`.

# Details

- Added `Gizmo3dFrameRenderSnapshot` as a named return type for the render-time model snapshot.
- Added `step_frame_animation(...)` to advance frame animation behind the binding.
- Added `frame_render_snapshot(...)` to build the scene target, selection, gizmo draw list, view
  projection, marquee, and depth snapshot behind the binding.
- Updated `record_engine_frame(...)` to call those binding methods and then continue GPU upload /
  immediate overlay work outside the model owner.
- Extended `gizmo3d_demo_surface.rs` to require those binding calls and reject the old direct
  frame-render model updates.

# Verification

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_frame_render_mutations_through_binding --no-fail-fast`
  failed because frame rendering was not routed through binding methods.
- `cargo nextest run -p fret-examples --test gizmo3d_demo_surface gizmo3d_demo_routes_frame_render_mutations_through_binding --no-fail-fast`

# Next Action

Re-run the raw-surface scan from latest `main` and choose the next demo/component family with
remaining app-side raw model exposure.

# Citations

- [gizmo3d_demo.rs](../../../../apps/fret-examples/src/gizmo3d_demo.rs)
- [gizmo3d_demo_surface.rs](../../../../apps/fret-examples/tests/gizmo3d_demo_surface.rs)
