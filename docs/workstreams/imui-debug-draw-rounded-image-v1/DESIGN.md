# ImUi Debug Draw Rounded Image v1

Status: Closed narrow product follow-on
Last updated: 2026-05-05

This lane adds Dear ImGui `AddImageRounded`-style clipping semantics to the canvas-backed IMUI
debug-draw helper. The Fret surface exposes rounded full-image and rounded image-region commands,
then lowers them through existing scene image primitives wrapped in a rounded-rect clip.

## Ownership

- `fret-ui-kit::imui` owns the debug-draw facade commands and typed corner-flag policy.
- `crates/fret-core` owns `SceneOp::Image`, `SceneOp::ImageRegion`, and `SceneOp::PushClipRRect`.
- `crates/fret-ui` owns Canvas painting and scene emission.
- `fret-imui`, runtime, resource loading, and renderer crates remain unchanged.

## Must-Be-True Outcomes

- Callers can record a rounded full-image draw with `add_image_rounded`.
- Callers can record a rounded image-region draw with `add_image_region_rounded`.
- Rounded image helpers use the same `DebugDrawRoundCorners` flags and `PathRect`-compatible
  rounding clamp as the debug-draw path rectangle helper.
- Degenerate, transparent, invalid-UV, or non-finite rounded image inputs record no scene work.
- Lowering reuses `PushClipRRect` plus existing `Image` / `ImageRegion` ops instead of introducing a
  texture mesh, image-loading recipe, or renderer contract.

## Non-Goals

- No image loading or resource lifetime ownership in `fret-ui-kit::imui`.
- No Dear ImGui texture tint RGB parity; current Fret image scene ops expose opacity, not image tint.
- No arbitrary `AddImageQuad` UV mesh.
- No `AddRectFilledMultiColor`; that needs vertex-colored primitive support rather than a linear
  gradient approximation.
- No hit-test-aware debug interaction.
