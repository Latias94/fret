# ImUi Debug Draw Vertex Quad v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Dear ImGui's `ImDrawList` has two debug-draw primitives that cannot be expressed correctly by
Fret's previous axis-aligned scene surface:

- `AddRectFilledMultiColor`, which emits a quad split into two triangles with one color per corner.
- `AddImageQuad`, which emits a quad with arbitrary corner positions and arbitrary UVs.

This lane adds those capabilities as renderer-owned scene mechanisms and exposes them through the
existing `fret-ui-kit::imui` debug-draw facade.

## Assumptions

- Confident: `fret-ui-kit::imui` is the correct facade layer because current debug draw helpers
  already live there.
- Confident: `crates/fret-core` must own the portable primitive vocabulary; approximating
  multi-color rects with `LinearGradient` would not match Dear ImGui's per-vertex interpolation.
- Confident: `crates/fret-render-wgpu` must own GPU encoding and pipeline behavior because both
  primitives are vertex-level draw operations.
- Likely: future DrawList channel splitting and command metadata should remain separate follow-ons
  because this slice only closes quad-level geometry.

## Ownership

- `crates/fret-core` owns `SceneOp::VertexColorQuad`, `SceneOp::ImageQuad`, and `UvPoint`.
- `crates/fret-render-wgpu` owns vertex encoding, shader inputs, the vertex-color pipeline, render
  plan flags, and draw recording.
- `ecosystem/fret-ui-kit::imui` owns the immediate debug-draw helper names and validity gates.
- `fret-imui` is not widened.

## Must-Be-True Outcomes

- A multi-color rect lowers to a single vertex-color scene primitive with corners ordered
  top-left, top-right, bottom-right, bottom-left.
- Vertex-color quads render as two triangles with indices `0,1,2` and `0,2,3`.
- Image quads support arbitrary points, arbitrary UVs, uniform tint, opacity, and sampling hint.
- Existing axis-aligned `Image` / `ImageRegion` and rounded-image helpers keep their semantics.
- Invalid or invisible IMUI commands are skipped before they reach the renderer.

## Non-Goals

- No generic `ImDrawList` channel splitting.
- No callback draw commands.
- No hit-test-aware debug interaction.
- No policy movement into `fret-imui` or `crates/fret-ui`.
