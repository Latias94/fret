# ImUi Debug Draw Vertex Quad v1 Evidence and Gates

Status: Closed.

## Evidence

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_draw.cpp`
- `crates/fret-core/src/scene/mod.rs`
- `crates/fret-core/src/scene/fingerprint.rs`
- `crates/fret-core/src/scene/validate.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/vertex_color.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/image.rs`
- `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/viewport.wgsl`
- `crates/fret-render-wgpu/src/renderer/tests.rs`
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`
- `docs/adr/0002-display-list.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

## Gates

```bash
cargo fmt --package fret-core --package fret-render-wgpu --package fret-ui-kit -- --check
cargo nextest run -p fret-core scene_state_stack_conformance --no-fail-fast
cargo nextest run -p fret-render-wgpu 'renderer::tests::vertex_color_quad_encodes_two_triangles_with_corner_colors' --no-fail-fast
cargo nextest run -p fret-render-wgpu 'renderer::tests::image_quad_encodes_custom_points_uvs_and_tint' --no-fail-fast
cargo nextest run -p fret-render-wgpu 'renderer::tests::shaders_validate_for_webgpu' --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui debug_draw_list_records --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui debug_draw_options_default_to_clipped_canvas --no-fail-fast
python -m json.tool docs/workstreams/imui-debug-draw-vertex-quad-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python tools/check_layering.py
python tools/check_workstream_catalog.py
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
