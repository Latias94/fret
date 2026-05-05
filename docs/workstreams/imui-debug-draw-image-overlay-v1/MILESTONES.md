# ImUi Debug Draw Image Overlay v1 Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Command Surface

Closed on 2026-05-04.

- `DebugDrawImageOptions`
- `DebugDrawSvgOptions`
- `add_image`
- `add_image_region`
- `add_svg_image`
- `add_svg_mask_icon`

## M1 - Scene Lowering

Closed on 2026-05-04.

- `ImageId` overlays lower to `SceneOp::Image`.
- Image regions lower to `SceneOp::ImageRegion`.
- SVG images lower through `CanvasPainter::svg_image`.
- SVG mask icons lower through `CanvasPainter::svg_mask_icon`.

## M2 - Proof

Closed on 2026-05-04.

- Focused debug-draw tests pass.
- Public smoke compile test exercises image overlay API.
- Adapter seam and response-contract smoke tests still pass.
