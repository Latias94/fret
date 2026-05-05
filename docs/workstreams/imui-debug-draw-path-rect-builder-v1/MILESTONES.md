# ImUi Debug Draw Path Rect Builder v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M0 - Reference Decision

- Dear ImGui reference: `PathRect` in `repo-ref/imgui/imgui_draw.cpp`.
- Fret decision: use typed `DebugDrawRoundCorners` flags rather than raw `ImDrawFlags` values.

## M1 - Implementation Slice

- Added `ImUiDebugDrawPath::rect`.
- Added `ImUiDebugDrawPath::rect_with_rounding`.
- Added square and rounded path point append helpers.
- Reused existing arc sampling for rounded corners.

## M2 - Verification Slice

- Added unit tests for square paths, selected rounded corners, clamping, disabled corners, and
  invalid input no-op behavior.
- Added public smoke compile coverage.
- Ran focused and full `fret-ui-kit --features imui` gates.

## M3 - Closeout

- Updated repo workstream indexes and the IMUI gap audit.
- Closed this lane as a narrow follow-on.
