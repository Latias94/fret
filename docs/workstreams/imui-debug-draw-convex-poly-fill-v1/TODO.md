# ImUi Debug Draw Convex Poly Fill v1 TODO

Status: Closed
Last updated: 2026-05-05

## Implementation

- [x] Add `add_convex_poly_filled` to `ImUiDebugDrawList`.
- [x] Lower point lists to closed fill paths.
- [x] Ignore point lists with fewer than three points.
- [x] Keep convexity validation outside the helper.

## Verification

- [x] Add source-level unit coverage for command recording and path closure.
- [x] Add public smoke compile coverage through `imui_debug_draw_smoke.rs`.
- [x] Run focused and full `fret-ui-kit --features imui` gates.

## Documentation

- [x] Record the workstream and closeout.
- [x] Update roadmap, TODO tracker, workstream index, umbrella evidence, and gap audit.
