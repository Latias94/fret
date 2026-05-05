# ImUi Debug Draw Vertex Quad v1 Closeout Audit - 2026-05-05

Status: closed closeout record.

## What Shipped

- Added `SceneOp::VertexColorQuad` for true per-corner color interpolation.
- Added `SceneOp::ImageQuad` and `UvPoint` for arbitrary image quad points and UVs.
- Added WGPU encoding, render-plan flags, draw recording, and a vertex-color pipeline.
- Extended viewport vertices so image draws can carry a uniform tint color.
- Added IMUI debug draw helpers for multi-color filled rects and image quads.
- Kept policy in `fret-ui-kit::imui`; `fret-imui` and `crates/fret-ui` were not widened.

## Evidence

- `crates/fret-core/src/scene/mod.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/vertex_color.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/image.rs`
- `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/viewport.wgsl`
- `crates/fret-render-wgpu/src/renderer/tests.rs`
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_draw.cpp`
- `docs/adr/0002-display-list.md`

## Gates Run

```bash
cargo nextest run -p fret-render-wgpu 'renderer::tests::vertex_color_quad_encodes_two_triangles_with_corner_colors' --no-fail-fast
cargo nextest run -p fret-render-wgpu 'renderer::tests::image_quad_encodes_custom_points_uvs_and_tint' --no-fail-fast
cargo nextest run -p fret-render-wgpu 'renderer::tests::shaders_validate_for_webgpu' --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui debug_draw_list_records --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui debug_draw_options_default_to_clipped_canvas --no-fail-fast
```

## Residual Gaps

- DrawList channel splitting.
- Callback/user draw commands.
- Per-command metadata beyond scene order.
