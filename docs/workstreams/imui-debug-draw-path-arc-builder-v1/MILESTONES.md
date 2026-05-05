# ImUi Debug Draw Path Arc Builder v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M1 - Explicit circular arc helper

Exit criteria:

- `ImUiDebugDrawPath` exposes an `arc_to` helper.
- The helper appends sampled points from `a_min` to `a_max`.
- `segments == 0` uses the stable debug-draw default segment count.

Status: Closed.

## M2 - Fast 12-step circular arc helper

Exit criteria:

- `ImUiDebugDrawPath` exposes an `arc_to_fast` helper.
- The helper uses Dear ImGui's 12-step circular angle vocabulary.
- Reverse arcs and degenerate positive radii have explicit coverage.

Status: Closed.

## M3 - Source and gate closeout

Exit criteria:

- Focused debug-draw gates pass.
- Full `fret-ui-kit --features imui` gate passes.
- The lane is indexed and remaining DrawList gaps stay separate.

Status: Closed.
