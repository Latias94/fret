# ImUi Debug Draw Vertex Quad v1 Milestones

Status: Closed.

## M0 - Contract Slice

Exit criteria:

- `SceneOp::VertexColorQuad` and `SceneOp::ImageQuad` exist in `fret-core`.
- Scene validation and fingerprinting account for both primitives.
- ADR 0002 names the new display-list surface.

Result: Complete.

## M1 - Renderer Slice

Exit criteria:

- WGPU encoding emits two triangles in Dear ImGui-compatible corner order.
- The viewport vertex format carries per-vertex color.
- A vertex-color pipeline renders solid vertex-color quads without texture binding.
- Image drawing still supports existing axis-aligned image and image-region paths.

Result: Complete.

## M2 - IMUI Facade Slice

Exit criteria:

- `ImUiDebugDrawList::add_rect_filled_multi_color` lowers to `SceneOp::VertexColorQuad`.
- Image quad helpers lower to `SceneOp::ImageQuad`.
- Focused tests and smoke compile coverage protect the helper surface.

Result: Complete.
