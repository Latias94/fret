# ImUi Debug Draw Path Bezier Builder v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M1 - Quadratic path Bezier helper

Exit criteria:

- `ImUiDebugDrawPath` exposes a quadratic curve-to helper.
- The helper starts from the last path point and appends sampled points.
- Missing start points do not panic or record commands.

Status: Closed.

## M2 - Cubic path Bezier helper

Exit criteria:

- `ImUiDebugDrawPath` exposes a cubic curve-to helper.
- The helper starts from the last path point and appends sampled points.
- `segments == 0` uses the stable debug-draw default segment count.

Status: Closed.

## M3 - Source and gate closeout

Exit criteria:

- Focused debug-draw gates pass.
- Full `fret-ui-kit --features imui` gate passes.
- The lane is indexed and remaining DrawList gaps stay separate.

Status: Closed.
