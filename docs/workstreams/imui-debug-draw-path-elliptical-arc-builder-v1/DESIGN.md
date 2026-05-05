# ImUi Debug Draw Path Elliptical Arc Builder v1

Status: Closed narrow product follow-on
Last updated: 2026-05-05

This lane extends the scoped IMUI debug-draw path builder with a Dear ImGui
`PathEllipticalArcTo`-style helper. The helper appends sampled rotated ellipse-arc points directly
to the temporary path, and finished paths still lower through the existing polyline or convex-fill
commands.

## Ownership

- `fret-ui-kit::imui` owns the debug-draw path-builder API.
- `crates/fret-core` owns the vector path command vocabulary.
- `crates/fret-ui` owns Canvas path painting.
- `fret-imui` remains unchanged.

## Must-Be-True Outcomes

- Callers can append sampled elliptical arcs with x/y radii, rotation, start/end radians, and a
  segment count.
- `segments == 0` uses a stable debug-draw default segment count instead of adaptive tessellation.
- Invalid radii, rotation, or angle inputs are no-op.
- Finished paths keep using the existing `stroke`, `stroke_with_style`, and `fill_convex` finishers.
- The helper does not introduce retained path state, renderer tessellation policy, or a new runtime
  contract.

## Non-Goals

- No rounded `PathRect` parity in this lane.
- No native curve-command path builder in this lane.
- No draw-list channel splitting.
- No per-command hit-testing.
- No renderer tessellation contract.
