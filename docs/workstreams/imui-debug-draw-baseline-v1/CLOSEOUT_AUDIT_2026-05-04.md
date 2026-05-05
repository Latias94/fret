# ImUi Debug Draw Baseline v1 Closeout Audit - 2026-05-04

Status: Closed.

This lane closes the first reusable IMUI debug-draw baseline by exposing a canvas-backed immediate
mode helper in `fret-ui-kit::imui`.

## What Shipped

- Added `debug_draw` and `debug_draw_with_options` facade helpers.
- Added `ImUiDebugDrawList` with line, rect, filled rect, and text commands.
- Lowered commands into declarative `Canvas` paint.
- Added smoke coverage for the facade and the new command list.

## Proof

- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke --no-fail-fast`
  passes.

## Remaining Work

Full Dear ImGui `DrawList` parity is still not closed. Shape primitives are covered by
`imui-debug-draw-shape-primitives-v1`, and stroke cap/join/dash policy is covered by
`imui-debug-draw-stroke-style-v1`; clip rect stack support is covered by
`imui-debug-draw-clip-stack-v1`; image overlays are covered by
`imui-debug-draw-image-overlay-v1`. Start separate follow-ons for per-command metadata, channel
splitting, hit-test-aware debug interaction, or image loading recipes.
