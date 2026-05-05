# ImUi Debug Draw Path Builder v1

Status: Closed narrow product follow-on
Last updated: 2026-05-05

This lane adds a scoped Dear ImGui `Path*`-style authoring helper to the canvas-backed IMUI
debug-draw surface. The API is intentionally temporary and closure-owned: callers build points
inside `draw.path(...)`, then finish with `stroke`, `stroke_with_style`, or `fill_convex`. Finishing
a path clears the temporary point list, matching Dear ImGui's `PathStroke` / `PathFillConvex`
ergonomics while lowering to existing Fret debug-draw commands.

## Ownership

- `fret-ui-kit::imui` owns the debug-draw command-list and path-builder API.
- `crates/fret-core` owns the vector path command vocabulary.
- `crates/fret-ui` owns Canvas path painting.
- `fret-imui` remains unchanged.

## Must-Be-True Outcomes

- Callers can build a temporary point path with `line_to`.
- Callers can avoid exact adjacent duplicates with `line_to_merge_duplicate`.
- `clear`, `point_count`, and `is_empty` expose minimal path-builder inspection and reset.
- `stroke` and `stroke_with_style` finish valid paths as existing polyline commands.
- `fill_convex` finishes valid paths as existing convex-fill commands.
- Finishing an invalid open stroke, closed stroke, or convex fill clears the temporary path without
  recording a draw command.
- The helper does not introduce retained path state, renderer tessellation policy, or a new runtime
  contract.

## Non-Goals

- No path arcs in this lane.
- No path Bezier builder helpers in this lane.
- No rounded `PathRect` parity in this lane.
- No retained path object across frames.
- No draw-list channel splitting.
- No per-command hit-testing.
- No renderer tessellation contract.
