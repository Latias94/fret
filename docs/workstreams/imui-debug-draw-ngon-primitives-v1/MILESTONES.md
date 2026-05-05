# ImUi Debug Draw Ngon Primitives v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M1 - Stroked ngon command

Exit criteria:

- `ImUiDebugDrawList` exposes a stroked regular-polygon helper.
- The command stores center, radius, and explicit segment count.
- Stroke style policy reuses `DebugDrawStrokeStyle`.

Status: Closed.

## M2 - Filled ngon command

Exit criteria:

- `ImUiDebugDrawList` exposes a filled regular-polygon helper.
- The command lowers to the same closed generated path with fill style.
- Invalid segment/radius inputs do not emit paint.

Status: Closed.

## M3 - Source and gate closeout

Exit criteria:

- Focused debug-draw gates pass.
- Full `fret-ui-kit --features imui` gate passes.
- The lane is indexed and remaining DrawList gaps stay separate.

Status: Closed.
