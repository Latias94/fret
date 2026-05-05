# ImUi Debug Draw Path Rect Builder v1

Status: Closed narrow product follow-on
Last updated: 2026-05-05

This lane extends the scoped IMUI debug-draw path builder with Dear ImGui `PathRect`-style square
and rounded rectangle helpers. The helper appends sampled points to the temporary path, and finished
paths still lower through the existing polyline or convex-fill commands.

## Ownership

- `fret-ui-kit::imui` owns the debug-draw path-builder API and typed round-corner flags.
- `crates/fret-core` owns geometry and vector path command vocabulary.
- `crates/fret-ui` owns Canvas path painting.
- `fret-imui`, runtime, and renderer crates remain unchanged.

## Must-Be-True Outcomes

- Callers can append a square rectangle path from a `Rect`.
- Callers can append a rounded rectangle path with selected rounded corners.
- The API uses Fret typed `DebugDrawRoundCorners` flags instead of leaking Dear ImGui raw
  `ImDrawFlags` bit layout.
- Rounded rectangles use the existing arc sampling vocabulary and clamp radius like Dear ImGui's
  `PathRect`.
- Invalid or empty rectangles are no-op, and finished paths keep using the existing
  `stroke`, `stroke_with_style`, and `fill_convex` finishers.

## Non-Goals

- No direct `AddRect` / `AddRectFilled` rounded command parity in this lane.
- No draw-list channel splitting.
- No per-command hit-testing.
- No renderer tessellation contract.
- No raw Dear ImGui flag ABI.
