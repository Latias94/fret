# ImUi Debug Draw Quad Primitives v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M1 - Stroked quad command

Exit criteria:

- `ImUiDebugDrawList` exposes a stroked quad helper.
- The command stores four caller-provided ordered points.
- Stroke style policy reuses `DebugDrawStrokeStyle`.

Status: Closed.

## M2 - Filled quad command

Exit criteria:

- `ImUiDebugDrawList` exposes a filled quad helper.
- The command lowers to the same closed four-point path with fill style.
- No triangulation or renderer contract is introduced.

Status: Closed.

## M3 - Source and gate closeout

Exit criteria:

- Focused debug-draw gates pass.
- Full `fret-ui-kit --features imui` gate passes.
- The lane is indexed and remaining DrawList gaps stay separate.

Status: Closed.
