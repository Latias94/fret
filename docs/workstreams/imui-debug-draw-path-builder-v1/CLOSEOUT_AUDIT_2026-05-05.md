# ImUi Debug Draw Path Builder v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane closes the first scoped Dear ImGui `Path*` ergonomics follow-on above the canvas-backed
IMUI debug-draw helper.

## What Shipped

- Added `ImUiDebugDrawList::path(...)`.
- Added `ImUiDebugDrawPath` with `line_to`, `line_to_merge_duplicate`, `clear`, `point_count`, and
  `is_empty`.
- Added `stroke`, `stroke_with_style`, and `fill_convex` finishers.
- Lowered valid finished paths to existing `Polyline` and `ConvexPolyFilled` commands.
- Cleared invalid finished paths without recording commands.
- Kept the implementation in `fret-ui-kit::imui` without widening `fret-imui`, runtime, renderer, or
  retained path contracts.

## Proof

- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-kit --features imui --no-fail-fast` passes.

## Remaining Work

Full Dear ImGui `ImDrawList` parity is still not closed. Start separate follow-ons for path arcs,
path Bezier builder helpers, rounded `PathRect` parity, channel splitting, hit-test-aware debug
interaction, reusable draw-list command metadata, or image loading recipes.
