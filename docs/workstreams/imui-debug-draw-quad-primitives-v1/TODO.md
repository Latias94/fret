# ImUi Debug Draw Quad Primitives v1 TODO

Status: Closed
Last updated: 2026-05-05

## Implementation

- [x] Add `add_quad` and `add_quad_with_style` to `ImUiDebugDrawList`.
- [x] Add `add_quad_filled` to `ImUiDebugDrawList`.
- [x] Lower four ordered points to a closed path for stroke and fill.
- [x] Keep validation and tessellation outside the helper.

## Verification

- [x] Add source-level unit coverage for command recording and path closure.
- [x] Add public smoke compile coverage through `imui_debug_draw_smoke.rs`.
- [x] Run focused and full `fret-ui-kit --features imui` gates.

## Documentation

- [x] Record the workstream and closeout.
- [x] Update roadmap, TODO tracker, workstream index, umbrella evidence, and gap audit.
