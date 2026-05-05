# ImUi Debug Draw Path Arc Builder v1 TODO

Status: Closed
Last updated: 2026-05-05

## Implementation

- [x] Add `arc_to` to `ImUiDebugDrawPath`.
- [x] Add `arc_to_fast` to `ImUiDebugDrawPath`.
- [x] Use a stable default segment count when `segments == 0`.
- [x] Treat invalid circular arc inputs as no-op.
- [x] Treat tiny positive radii as a center-point path segment.

## Verification

- [x] Add source-level unit coverage for sampled arc points, fast 12-step arcs, default segments,
  degenerate radii, and invalid inputs.
- [x] Add public smoke compile coverage through `imui_debug_draw_smoke.rs`.
- [x] Run focused and full `fret-ui-kit --features imui` gates.

## Documentation

- [x] Record the workstream and closeout.
- [x] Update roadmap, TODO tracker, workstream index, umbrella evidence, and gap audit.
