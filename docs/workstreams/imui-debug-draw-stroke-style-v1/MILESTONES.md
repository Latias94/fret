# ImUi Debug Draw Stroke Style v1 Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Public Style Surface

Closed on 2026-05-04.

- `DebugDrawStrokeStyle`
- `add_line_with_style`
- `add_polyline_with_style`
- `add_rect_with_style`
- `add_triangle_with_style`
- `add_circle_with_style`

## M1 - PathStyle Lowering

Closed on 2026-05-04.

- Default width-only style keeps `PathStyle::Stroke`.
- Explicit cap/join/miter/dash policy uses `PathStyle::StrokeV2`.
- Invalid dash and miter inputs are filtered.

## M2 - Proof

Closed on 2026-05-04.

- Focused debug-draw tests pass.
- Public smoke compile test exercises the styled API.
- Adapter seam and response-contract smoke tests still pass.
