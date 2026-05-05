# ImUi Debug Draw Shape Primitives v1 Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Shape Surface

Closed on 2026-05-04.

- `ImUiDebugDrawList::add_polyline`
- `ImUiDebugDrawList::add_triangle`
- `ImUiDebugDrawList::add_triangle_filled`
- `ImUiDebugDrawList::add_circle`
- `ImUiDebugDrawList::add_circle_filled`

## M1 - Canvas Lowering

Closed on 2026-05-04.

- Open/closed polylines lower to path stroke commands.
- Triangles lower to path stroke/fill commands.
- Circles lower to four cubic arcs and close.
- Degenerate shapes and transparent/zero-size commands are skipped.

## M2 - Proof

Closed on 2026-05-04.

- Focused debug-draw tests pass.
- Facade seam and response-contract smoke tests still pass.
