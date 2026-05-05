# ImUi Debug Draw Bezier Primitives v1 TODO

Status: Closed
Last updated: 2026-05-05

## Implementation

- [x] Add quadratic Bezier commands to `ImUiDebugDrawList`.
- [x] Add cubic Bezier commands to `ImUiDebugDrawList`.
- [x] Preserve thickness-based and styled variants.
- [x] Lower to `PathCommand::QuadTo` and `PathCommand::CubicTo`.

## Verification

- [x] Add source-level unit coverage for command recording and native path lowering.
- [x] Add public smoke compile coverage through `imui_debug_draw_smoke.rs`.
- [x] Keep adapter seam and response contract gates unchanged.

## Documentation

- [x] Record the workstream and closeout.
- [x] Update roadmap, TODO tracker, workstream index, umbrella evidence, and gap audit.
