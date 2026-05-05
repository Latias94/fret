# ImUi Debug Draw Ellipse Primitives v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M1 - Stroked ellipse command

Exit criteria:

- `ImUiDebugDrawList` exposes a stroked ellipse helper.
- The command stores center, radii, rotation, and segment count.
- Stroke style policy reuses `DebugDrawStrokeStyle`.

Status: Closed.

## M2 - Filled ellipse command

Exit criteria:

- `ImUiDebugDrawList` exposes a filled ellipse helper.
- The command lowers to the same generated path with fill style.
- Invalid segment/radius/rotation inputs do not emit paint.

Status: Closed.

## M3 - Source and gate closeout

Exit criteria:

- Focused debug-draw gates pass.
- Full `fret-ui-kit --features imui` gate passes.
- The lane is indexed and remaining DrawList gaps stay separate.

Status: Closed.
