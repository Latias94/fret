# ImUi Debug Draw Convex Poly Fill v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M1 - Filled polygon command

Exit criteria:

- `ImUiDebugDrawList` exposes a convex filled polygon helper.
- The command stores an app-provided ordered point list.
- The command is public-smoke compiled.

Status: Closed.

## M2 - Closed fill path lowering

Exit criteria:

- Valid point lists lower to a closed path with fill style.
- Short point lists are ignored.
- No triangulation or renderer contract is introduced.

Status: Closed.

## M3 - Source and gate closeout

Exit criteria:

- Focused debug-draw gates pass.
- Full `fret-ui-kit --features imui` gate passes.
- The lane is indexed and remaining DrawList gaps stay separate.

Status: Closed.
