# ImUi Debug Draw Ellipse Primitives v1 TODO

Status: Closed
Last updated: 2026-05-05

## Implementation

- [x] Add `add_ellipse` and `add_ellipse_with_style` to `ImUiDebugDrawList`.
- [x] Add `add_ellipse_filled` to `ImUiDebugDrawList`.
- [x] Lower center/radii/rotation/segment inputs to a closed path.
- [x] Ignore invalid segments, radii, and rotation.

## Verification

- [x] Add source-level unit coverage for command recording and path generation.
- [x] Add public smoke compile coverage through `imui_debug_draw_smoke.rs`.
- [x] Run focused and full `fret-ui-kit --features imui` gates.

## Documentation

- [x] Record the workstream and closeout.
- [x] Update roadmap, TODO tracker, workstream index, umbrella evidence, and gap audit.
