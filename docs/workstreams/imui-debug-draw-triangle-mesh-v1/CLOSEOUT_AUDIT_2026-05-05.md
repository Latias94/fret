# ImUi Debug Draw Triangle Mesh v1 Closeout Audit - 2026-05-05

Status: closed closeout record.

## What Shipped

- Added copyable `SceneMeshVertex` to the core scene contract.
- Added `SceneOp::VertexColorTriangle` and `SceneOp::ImageTriangle`.
- Added validation, fingerprinting, and stack conformance coverage for the new triangle ops.
- Added WGPU encoding for vertex-color and textured triangle primitives.
- Added IMUI `DebugDrawVertex`, `add_triangle_list`, `add_triangle_mesh`,
  `add_image_triangle_mesh`, and `add_image_triangle_mesh_with_options`.
- Kept the Dear ImGui raw mesh parity step bounded: no renderer callbacks, raw buffer ownership, or
  non-copyable scene ops.

## Evidence

- `crates/fret-core/src/scene/mod.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/vertex_color.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/image.rs`
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_draw.cpp`

## Gates Run

```bash
cargo check -p fret-core
cargo check -p fret-render-wgpu
cargo check -p fret-ui-kit --features imui
```

Full closeout gates are recorded in `EVIDENCE_AND_GATES.md`.

## Residual Gaps

- Large mesh batching is not optimized; IMUI lowers indexed meshes to fixed triangle scene ops.
- There is no public writable vertex/index buffer API equivalent to `PrimReserve`.
- Callback/user draw commands remain intentionally out of the generic scene contract.
