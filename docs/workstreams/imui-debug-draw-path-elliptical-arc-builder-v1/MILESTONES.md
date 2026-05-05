# ImUi Debug Draw Path Elliptical Arc Builder v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M1 - Rotated elliptical arc helper

Exit criteria:

- `ImUiDebugDrawPath` exposes an `elliptical_arc_to` helper.
- The helper appends sampled points from `a_min` to `a_max`.
- Rotation and x/y radii are applied before appending points.

Status: Closed.

## M2 - Validation and default segments

Exit criteria:

- `segments == 0` uses the stable debug-draw default segment count.
- Invalid radii, rotation, and angle inputs are no-op.
- Unit tests cover both exact unrotated samples and rotated samples.

Status: Closed.

## M3 - Source and gate closeout

Exit criteria:

- Focused debug-draw gates pass.
- Full `fret-ui-kit --features imui` gate passes.
- The lane is indexed and remaining DrawList gaps stay separate.

Status: Closed.
