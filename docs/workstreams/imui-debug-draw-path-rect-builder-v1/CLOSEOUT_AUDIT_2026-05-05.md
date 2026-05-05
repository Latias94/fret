# ImUi Debug Draw Path Rect Builder v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane closes the scoped Dear ImGui `PathRect` ergonomics follow-on above the canvas-backed IMUI
debug-draw helper.

## What Shipped

- Added typed `DebugDrawRoundCorners` flags.
- Added `ImUiDebugDrawPath::rect`.
- Added `ImUiDebugDrawPath::rect_with_rounding`.
- Appended square rectangle points for unrounded or disabled-corner paths.
- Appended sampled corner arcs for rounded rectangle paths.
- Clamped rounding radius using Dear ImGui's `PathRect` outcome.
- Kept the implementation in `fret-ui-kit::imui` without widening `fret-imui`, runtime, renderer,
  retained path, or hit-testing contracts.

## Proof

- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-kit --features imui --no-fail-fast` passes.

## Remaining Work

Full Dear ImGui `ImDrawList` parity is still not closed. Start separate follow-ons for channel
splitting, hit-test-aware debug interaction, reusable draw-list command metadata, image loading
recipes, or concave path fill parity.
