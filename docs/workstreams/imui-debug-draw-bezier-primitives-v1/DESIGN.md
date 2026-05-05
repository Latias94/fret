# ImUi Debug Draw Bezier Primitives v1

Status: Closed narrow product follow-on
Last updated: 2026-05-05

This lane adds Dear ImGui-style quadratic and cubic Bezier primitives to the canvas-backed IMUI
debug-draw helper. It uses the existing Fret vector path mechanism and keeps the immediate facade
as a command emitter rather than a renderer or resource owner.

## Ownership

- `fret-ui-kit::imui` owns the debug-draw command-list API and policy-level convenience methods.
- `crates/fret-core` already owns `PathCommand::QuadTo` and `PathCommand::CubicTo`.
- `crates/fret-ui` remains the Canvas mechanism layer.
- `fret-imui` stays thin and unchanged.

## Must-Be-True Outcomes

- Callers can emit a stroked quadratic Bezier curve.
- Callers can emit a stroked cubic Bezier curve.
- Thickness-based calls remain available for the simple Dear ImGui-shaped path.
- Styled calls reuse `DebugDrawStrokeStyle` for cap/join/miter/dash policy.
- Lowering uses native path commands, not flattened ad hoc polyline approximation.

## Non-Goals

- No retained path builder API.
- No `PathStroke` / `PathFillConvex` stack.
- No draw-list channel splitting.
- No per-command hit-testing.
- No renderer contract changes.
