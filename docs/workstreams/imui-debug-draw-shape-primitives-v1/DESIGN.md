# ImUi Debug Draw Shape Primitives v1

Status: Closed narrow product follow-on
Last updated: 2026-05-04

This lane extends the closed `imui-debug-draw-baseline-v1` helper with the smallest useful
Dear ImGui-style shape floor: polylines, triangles, and circles.

## Ownership

- `fret-ui-kit::imui` owns the facade command list and the canvas-backed lowering.
- `crates/fret-ui` remains the mechanism layer through the existing `Canvas` path paint surface.
- `fret-imui` stays thin and does not grow renderer or draw-list ownership.

## Must-Be-True Outcomes

- Callers can draw open or closed polylines.
- Callers can draw stroked and filled triangles.
- Callers can draw stroked and filled circles.
- Degenerate shapes are skipped before paint emission.
- The implementation uses existing path/scene mechanisms rather than adding runtime or renderer APIs.

## Non-Goals

- No full Dear ImGui `ImDrawList` parity.
- Stroke cap/join/dash policy is owned by `imui-debug-draw-stroke-style-v1`.
- Clip rect stack support is owned by `imui-debug-draw-clip-stack-v1`.
- No image overlays or channel splitting.
- No interaction or hit-test routing for debug-draw commands.
