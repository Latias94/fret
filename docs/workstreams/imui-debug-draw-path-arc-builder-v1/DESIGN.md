# ImUi Debug Draw Path Arc Builder v1

Status: Closed narrow product follow-on
Last updated: 2026-05-05

This lane extends the scoped IMUI debug-draw path builder with Dear ImGui `PathArcTo`- and
`PathArcToFast`-style circular arc helpers. The helpers append sampled points directly to the
temporary path, and finished paths still lower through the existing polyline or convex-fill
commands.

## Ownership

- `fret-ui-kit::imui` owns the debug-draw path-builder API.
- `crates/fret-core` owns the vector path command vocabulary.
- `crates/fret-ui` owns Canvas path painting.
- `fret-imui` remains unchanged.

## Must-Be-True Outcomes

- Callers can append sampled circular arcs with explicit start/end radians and a segment count.
- Callers can append 12-step fast circular arcs with Dear ImGui's `0 == east`, `3 == south`,
  `6 == west`, `9 == north`, `12 == east` vocabulary.
- `segments == 0` uses a stable debug-draw default segment count instead of adaptive tessellation.
- Invalid radius and angle inputs are no-op.
- Very small positive radii degrade to the center point.
- Finished paths keep using the existing `stroke`, `stroke_with_style`, and `fill_convex` finishers.
- The helper does not introduce retained path state, renderer tessellation policy, or a new runtime
  contract.

## Non-Goals

- No elliptical path arcs in this lane.
- No rounded `PathRect` parity in this lane.
- No native curve-command path builder in this lane.
- No draw-list channel splitting.
- No per-command hit-testing.
- No renderer tessellation contract.
