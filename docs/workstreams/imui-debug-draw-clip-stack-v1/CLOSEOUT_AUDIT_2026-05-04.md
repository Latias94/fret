# ImUi Debug Draw Clip Stack v1 Closeout Audit - 2026-05-04

Status: Closed.

This lane closes the first clip-stack slice for the canvas-backed IMUI debug-draw helper.

## What Shipped

- Added `push_clip_rect` and `pop_clip_rect` to `ImUiDebugDrawList`.
- Lowered clip commands to existing scene clip operations.
- Ignored empty clip rects and unmatched pops.
- Auto-balanced unclosed debug clips at the end of the paint pass.
- Extended command-list and public smoke compile coverage.

## Proof

- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke --no-fail-fast`
  passes.

## Remaining Work

Full Dear ImGui `ImDrawList` parity is still not closed. Image overlays are now closed by
`docs/workstreams/imui-debug-draw-image-overlay-v1/CLOSEOUT_AUDIT_2026-05-04.md`. Start separate
follow-ons for channel splitting, hit-test-aware debug interaction, reusable draw-list command
metadata, or image loading recipes.
