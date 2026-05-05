# ImUi Debug Draw Path Rect Builder v1 TODO

Status: Closed
Last updated: 2026-05-05

## Implementation

- [x] Add typed `DebugDrawRoundCorners` flags.
- [x] Add `rect` and `rect_with_rounding` to `ImUiDebugDrawPath`.
- [x] Append square rectangle points for unrounded or corner-disabled paths.
- [x] Append sampled corner arcs for rounded rectangle paths.
- [x] Clamp rounded rectangle radius using the Dear ImGui `PathRect` outcome.

## Verification

- [x] Add source-level unit coverage for square paths, selected rounded corners, radius clamping,
  corner-disabled square fallback, and invalid inputs.
- [x] Add public smoke compile coverage through `imui_debug_draw_smoke.rs`.
- [x] Run focused and full `fret-ui-kit --features imui` gates.

## Documentation

- [x] Record the workstream and closeout.
- [x] Update roadmap, TODO tracker, workstream index, umbrella evidence, and gap audit.
