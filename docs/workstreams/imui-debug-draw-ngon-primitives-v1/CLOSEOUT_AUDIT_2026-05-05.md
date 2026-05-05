# ImUi Debug Draw Ngon Primitives v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane closes the next explicit-segment regular-polygon follow-on above the canvas-backed IMUI
debug-draw helper.

## What Shipped

- Added `add_ngon` and `add_ngon_with_style`.
- Added `add_ngon_filled`.
- Lowered valid center/radius/segment inputs to closed Canvas paths for stroke and fill.
- Ignored fewer than three segments and non-positive radii.
- Kept the implementation in `fret-ui-kit::imui` without widening `fret-imui`, runtime, or renderer
  contracts.

## Proof

- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-kit --features imui --no-fail-fast` passes.

## Remaining Work

Full Dear ImGui `ImDrawList` parity is still not closed. Start separate follow-ons for ellipse,
path-builder ergonomics, channel splitting, hit-test-aware debug interaction, reusable draw-list
command metadata, or image loading recipes.
