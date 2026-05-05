# ImUi Debug Draw Bezier Primitives v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M1 - Bezier command surface

Exit criteria:

- `ImUiDebugDrawList` exposes quadratic and cubic Bezier helpers.
- Existing thickness-based authoring remains source-compatible.
- Styled variants accept `DebugDrawStrokeStyle`.

Status: Closed.

## M2 - Native path lowering

Exit criteria:

- Quadratic curves lower to `PathCommand::QuadTo`.
- Cubic curves lower to `PathCommand::CubicTo`.
- Tests prove the lowering does not flatten curves into polyline commands.

Status: Closed.

## M3 - Source and gate closeout

Exit criteria:

- Public smoke compile coverage uses the new helpers.
- Focused debug-draw gates pass.
- The lane is indexed and the remaining DrawList gaps are explicitly deferred.

Status: Closed.
