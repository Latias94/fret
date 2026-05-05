# ImUi Debug Draw Clip Stack v1

Status: Closed narrow product follow-on
Last updated: 2026-05-04

This lane adds a small Dear ImGui-style clipping stack to the canvas-backed IMUI debug-draw helper.
The implementation lowers to existing scene clip operations and does not add runtime or renderer
contracts.

## Ownership

- `fret-ui-kit::imui` owns the debug-draw command-list API.
- `crates/fret-core` already owns scene clip operations.
- `crates/fret-ui` remains the mechanism layer through `CanvasPainter`.

## Must-Be-True Outcomes

- `ImUiDebugDrawList` exposes `push_clip_rect` and `pop_clip_rect`.
- Empty clip rects are ignored.
- Extra pops with no matching push are ignored.
- Unclosed debug-draw clips are popped at the end of the paint pass.
- Existing `DebugDrawOptions::clip_to_bounds` remains the whole-canvas outer clip policy.

## Non-Goals

- No rounded clip stack.
- No channel splitting.
- No per-command hit-testing.
