# ImUi Debug Draw Image Overlay v1

Status: Closed narrow product follow-on
Last updated: 2026-05-04

This lane adds image-overlay commands to the canvas-backed IMUI debug-draw helper. It intentionally
draws already-registered or explicitly supplied image/SVG sources; resource loading remains outside
the immediate facade.

## Ownership

- `fret-ui-kit::imui` owns the debug-draw command-list API.
- Image/SVG registration and lifetime stay owned by existing app/runtime resource mechanisms.
- `crates/fret-ui` remains the mechanism layer through `CanvasPainter` and scene ops.

## Must-Be-True Outcomes

- Callers can draw a registered `ImageId`.
- Callers can draw an `ImageId` region using `UvRect`.
- Callers can draw an SVG image from `SvgSource`.
- Callers can draw a tinted SVG mask icon from `SvgSource`.
- Opacity and UV inputs are sanitized before paint emission.

## Non-Goals

- No image loading API.
- No texture atlas ownership.
- No draw-list channel splitting.
- No per-command hit-testing.
