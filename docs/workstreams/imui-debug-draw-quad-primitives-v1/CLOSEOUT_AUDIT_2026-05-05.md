# ImUi Debug Draw Quad Primitives v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane closes the next basic-shape follow-on above the canvas-backed IMUI debug-draw helper.

## What Shipped

- Added `add_quad` and `add_quad_with_style`.
- Added `add_quad_filled`.
- Lowered four ordered points to a closed Canvas path for stroke and fill.
- Kept validation and tessellation caller-owned.
- Kept the implementation in `fret-ui-kit::imui` without widening `fret-imui`, runtime, or renderer
  contracts.

## Proof

- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-kit --features imui --no-fail-fast` passes.

## Remaining Work

Full Dear ImGui `ImDrawList` parity is still not closed. Start separate follow-ons for ngon,
ellipse, path-builder ergonomics, channel splitting, hit-test-aware debug interaction, reusable
draw-list command metadata, or image loading recipes.
