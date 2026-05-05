# ImUi Debug Draw Path Builder v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M1 - Scoped line path builder

Exit criteria:

- `ImUiDebugDrawList` exposes a `path(...)` closure helper.
- `ImUiDebugDrawPath` supports `line_to`, duplicate merging, clearing, and minimal inspection.
- Builder state is temporary and cannot outlive the closure.

Status: Closed.

## M2 - Stroke and fill finishers

Exit criteria:

- Valid open and closed strokes lower to existing polyline commands.
- Valid convex fills lower to existing convex-fill commands.
- Invalid finished paths clear without recording commands.

Status: Closed.

## M3 - Source and gate closeout

Exit criteria:

- Focused debug-draw gates pass.
- Full `fret-ui-kit --features imui` gate passes.
- The lane is indexed and remaining DrawList gaps stay separate.

Status: Closed.
