# ImUi Debug Draw Stroke Style v1

Status: Closed narrow product follow-on
Last updated: 2026-05-04

This lane adds explicit stroke policy to the canvas-backed IMUI debug-draw helper. It builds on the
closed shape-primitives lane and uses existing `PathStyle::StrokeV2` support for cap/join/dash
without changing runtime or renderer contracts.

## Ownership

- `fret-ui-kit::imui` owns `DebugDrawStrokeStyle` and the facade convenience methods.
- `crates/fret-core` owns the stroke vocabulary (`StrokeCapV1`, `StrokeJoinV1`, `DashPatternV1`,
  `StrokeStyleV2`).
- `crates/fret-ui` and the renderer remain unchanged; debug draw lowers through existing `Canvas`.

## Must-Be-True Outcomes

- Existing `thickness: Px` APIs keep compiling.
- Styled variants exist for line, polyline, rect, triangle, and circle commands.
- Default style keeps the old `PathStyle::Stroke` path.
- Explicit cap/join/dash policy lowers to `PathStyle::StrokeV2`.
- Invalid dash and miter inputs are ignored before reaching the renderer.

## Non-Goals

- No per-command hit-testing.
- No image overlays or texture payloads.
- No draw-list channel splitting or nested clipping stack API.
