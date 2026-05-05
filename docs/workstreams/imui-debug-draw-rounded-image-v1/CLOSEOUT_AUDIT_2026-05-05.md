# ImUi Debug Draw Rounded Image v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane closes the Dear ImGui `AddImageRounded`-style rounded image clipping follow-on above the
canvas-backed IMUI debug-draw helper.

## What Shipped

- Added `ImUiDebugDrawList::add_image_rounded`.
- Added `ImUiDebugDrawList::add_image_rounded_with_options`.
- Added `ImUiDebugDrawList::add_image_region_rounded`.
- Added dedicated rounded image and rounded image-region command variants.
- Lowered rounded image commands through `SceneOp::PushClipRRect` plus existing `Image` /
  `ImageRegion` scene ops.
- Reused the same `PathRect`-compatible rounding clamp as the path rectangle builder.
- Kept image loading, image tinting, arbitrary image quads, and renderer mesh contracts out of this
  IMUI facade slice.

## Proof

- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-kit --features imui --no-fail-fast` passes.

## Remaining Work

Full Dear ImGui `ImDrawList` parity is still not closed. Start separate follow-ons for channel
splitting, hit-test-aware debug interaction, reusable draw-list command metadata, image loading
recipes, image tinting, arbitrary image quads, or multi-color rect fill. `AddRectFilledMultiColor`
should wait for an honest vertex-colored primitive or mesh contract; a single linear gradient is not
semantically equivalent.
