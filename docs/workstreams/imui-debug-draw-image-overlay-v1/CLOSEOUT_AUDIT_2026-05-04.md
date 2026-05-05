# ImUi Debug Draw Image Overlay v1 Closeout Audit - 2026-05-04

Status: Closed.

This lane closes the first image-overlay slice for the canvas-backed IMUI debug-draw helper.

## What Shipped

- Added `DebugDrawImageOptions` and `DebugDrawSvgOptions`.
- Added `add_image` and `add_image_region` for already-registered `ImageId`s.
- Added `add_svg_image` and `add_svg_mask_icon` for explicit `SvgSource`s.
- Lowered image/SVG commands through existing scene and canvas mechanisms.
- Added opacity and UV validation helpers.
- Extended command-list and public smoke compile coverage.

## Proof

- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke --no-fail-fast`
  passes.

## Remaining Work

Full Dear ImGui `ImDrawList` parity is still not closed. Start separate follow-ons for channel
splitting, hit-test-aware debug interaction, reusable draw-list command metadata, or image loading
recipes.
