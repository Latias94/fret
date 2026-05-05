# ImUi Debug Draw Path Bezier Builder v1 TODO

Status: Closed
Last updated: 2026-05-05

## Implementation

- [x] Add `bezier_quadratic_curve_to` to `ImUiDebugDrawPath`.
- [x] Add `bezier_cubic_curve_to` to `ImUiDebugDrawPath`.
- [x] Use the current last path point as the curve start point.
- [x] Treat missing start points as no-op instead of panicking.
- [x] Use a stable default segment count when `segments == 0`.

## Verification

- [x] Add source-level unit coverage for sampled quadratic/cubic points, missing start points, and
  default segments.
- [x] Add public smoke compile coverage through `imui_debug_draw_smoke.rs`.
- [x] Run focused and full `fret-ui-kit --features imui` gates.

## Documentation

- [x] Record the workstream and closeout.
- [x] Update roadmap, TODO tracker, workstream index, umbrella evidence, and gap audit.
