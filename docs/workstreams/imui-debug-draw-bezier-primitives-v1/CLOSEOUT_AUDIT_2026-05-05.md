# ImUi Debug Draw Bezier Primitives v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane closes the next richer-shape follow-on above the canvas-backed IMUI debug-draw helper.

## What Shipped

- Added `add_bezier_quadratic` and `add_bezier_quadratic_with_style`.
- Added `add_bezier_cubic` and `add_bezier_cubic_with_style`.
- Reused `DebugDrawStrokeStyle` for styled Bezier strokes.
- Lowered quadratic curves to `PathCommand::QuadTo`.
- Lowered cubic curves to `PathCommand::CubicTo`.
- Kept the implementation in `fret-ui-kit::imui` without widening `fret-imui`, runtime, or renderer
  contracts.

## Proof

- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke --no-fail-fast`
  passes.

## Remaining Work

Full Dear ImGui `ImDrawList` parity is still not closed. Start separate follow-ons for path-builder
ergonomics, channel splitting, hit-test-aware debug interaction, reusable draw-list command
metadata, or image loading recipes.
