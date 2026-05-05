# ImUi Debug Draw Shape Primitives v1 Closeout Audit - 2026-05-04

Status: Closed.

This lane closes the first richer-shape follow-on above the canvas-backed `debug_draw` baseline.

## What Shipped

- Added `add_polyline` with open/closed path support.
- Added stroked and filled triangle commands.
- Added stroked and filled circle commands backed by four cubic arcs.
- Kept all shape lowering inside `fret-ui-kit::imui` over the existing declarative `Canvas`.
- Added command-order, polyline, triangle, circle, and public smoke compile coverage.

## Proof

- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke --no-fail-fast`
  passes.

## Remaining Work

Full Dear ImGui `ImDrawList` parity is still not closed. Stroke cap/join/dash policy is covered by
`docs/workstreams/imui-debug-draw-stroke-style-v1/CLOSEOUT_AUDIT_2026-05-04.md`, and clip rect
stack support is covered by `docs/workstreams/imui-debug-draw-clip-stack-v1/CLOSEOUT_AUDIT_2026-05-04.md`;
image overlays are covered by
`docs/workstreams/imui-debug-draw-image-overlay-v1/CLOSEOUT_AUDIT_2026-05-04.md`. Start separate
follow-ons for channel splitting, hit-test-aware debug interaction, reusable draw-list command
metadata, or image loading recipes.
