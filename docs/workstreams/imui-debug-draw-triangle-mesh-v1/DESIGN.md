# ImUi Debug Draw Triangle Mesh v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Dear ImGui exposes `PrimReserve` / `PrimWriteVtx` / `PrimWriteIdx` for advanced custom geometry.
Fret should not mirror that by putting raw mutable buffers or user render callbacks into the
renderer-facing scene contract. The bounded equivalent for this lane is a fixed triangle scene
primitive plus IMUI helpers that can lower indexed triangle meshes into ordered scene triangles.

## Ownership

- `crates/fret-core` owns the portable triangle scene contract.
- `crates/fret-render-wgpu` owns default WGPU encoding for vertex-color and textured triangles.
- `fret-ui-kit::imui` owns the Dear ImGui-style authoring helper and command buffering.
- Renderer callbacks, draw command metadata, and batch-level raw buffers stay out of this lane.

## Must-Be-True Outcomes

- `SceneOp` stays copyable; existing scene caches and replay paths do not need a heap-owned op.
- Vertex-color and textured triangles validate finite position/UV/color data.
- The default WGPU renderer encodes exactly three vertices for a triangle op and preserves per-vertex
  color/UV data.
- IMUI debug draw can record indexed triangle meshes and image triangle meshes without exposing
  renderer internals.
- Invalid indices, degenerate triangles, non-finite vertices, and fully transparent triangles do not
  emit scene ops from the IMUI helper.

## Non-Goals

- No nested raw draw command buffer.
- No `AddCallback` / user renderer callback.
- No batching guarantee for large meshes.
- No renderer-owned mesh resource service yet.
