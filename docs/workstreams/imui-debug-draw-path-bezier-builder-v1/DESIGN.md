# ImUi Debug Draw Path Bezier Builder v1

Status: Closed narrow product follow-on
Last updated: 2026-05-05

This lane extends the scoped IMUI debug-draw path builder with Dear ImGui
`PathBezierQuadraticCurveTo`- and `PathBezierCubicCurveTo`-style helpers. The helpers append sampled
points to the current temporary path, starting from the last point already in the builder. The
finished path still lowers through the existing polyline or convex-fill commands.

## Ownership

- `fret-ui-kit::imui` owns the debug-draw path-builder API.
- `crates/fret-core` owns the vector path command vocabulary.
- `crates/fret-ui` owns Canvas path painting.
- `fret-imui` remains unchanged.

## Must-Be-True Outcomes

- Callers can append sampled quadratic Bezier curve points from the current path point.
- Callers can append sampled cubic Bezier curve points from the current path point.
- Calling either helper without a current path point is a no-op.
- `segments == 0` uses a stable debug-draw default segment count instead of introducing adaptive
  tessellation.
- Finished paths keep using the existing `stroke`, `stroke_with_style`, and `fill_convex` finishers.
- The helper does not introduce retained path state, renderer tessellation policy, or a new runtime
  contract.

## Non-Goals

- No path arcs in this lane.
- No elliptical path arcs in this lane.
- No rounded `PathRect` parity in this lane.
- No native curve-command path builder in this lane.
- No draw-list channel splitting.
- No per-command hit-testing.
- No renderer tessellation contract.
