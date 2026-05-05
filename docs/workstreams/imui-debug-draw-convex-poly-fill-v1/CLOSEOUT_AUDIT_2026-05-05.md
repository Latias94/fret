# ImUi Debug Draw Convex Poly Fill v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane closes the next filled-shape follow-on above the canvas-backed IMUI debug-draw helper.

## What Shipped

- Added `add_convex_poly_filled`.
- Lowered point lists to a closed Canvas fill path.
- Ignored point lists with fewer than three points.
- Kept convexity as the caller-owned contract implied by the helper name.
- Kept the implementation in `fret-ui-kit::imui` without widening `fret-imui`, runtime, or renderer
  contracts.

## Proof

- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-kit --features imui --no-fail-fast` passes.

## Remaining Work

Full Dear ImGui `ImDrawList` parity is still not closed. Start separate follow-ons for path-builder
ergonomics, channel splitting, hit-test-aware debug interaction, reusable draw-list command
metadata, or image loading recipes.
