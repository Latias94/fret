# ImUi Debug Draw Path Builder v1 TODO

Status: Closed
Last updated: 2026-05-05

## Implementation

- [x] Add a scoped `ImUiDebugDrawPath` builder to `ImUiDebugDrawList`.
- [x] Add `line_to`, `line_to_merge_duplicate`, `clear`, `point_count`, and `is_empty`.
- [x] Add `stroke` and `stroke_with_style` finishers that lower to existing polyline commands.
- [x] Add `fill_convex` finisher that lowers to existing convex-fill commands.
- [x] Clear invalid finished paths without recording commands.

## Verification

- [x] Add source-level unit coverage for command recording, duplicate merging, clearing, and invalid
  finishers.
- [x] Add public smoke compile coverage through `imui_debug_draw_smoke.rs`.
- [x] Run focused and full `fret-ui-kit --features imui` gates.

## Documentation

- [x] Record the workstream and closeout.
- [x] Update roadmap, TODO tracker, workstream index, umbrella evidence, and gap audit.
