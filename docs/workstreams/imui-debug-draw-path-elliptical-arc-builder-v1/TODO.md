# ImUi Debug Draw Path Elliptical Arc Builder v1 TODO

Status: Closed
Last updated: 2026-05-05

## Implementation

- [x] Add `elliptical_arc_to` to `ImUiDebugDrawPath`.
- [x] Append rotated ellipse-arc samples to the temporary point path.
- [x] Use a stable default segment count when `segments == 0`.
- [x] Treat invalid elliptical arc inputs as no-op.

## Verification

- [x] Add source-level unit coverage for sampled elliptical arc points, rotation, default segments,
  and invalid inputs.
- [x] Add public smoke compile coverage through `imui_debug_draw_smoke.rs`.
- [x] Run focused and full `fret-ui-kit --features imui` gates.

## Documentation

- [x] Record the workstream and closeout.
- [x] Update roadmap, TODO tracker, workstream index, umbrella evidence, and gap audit.
