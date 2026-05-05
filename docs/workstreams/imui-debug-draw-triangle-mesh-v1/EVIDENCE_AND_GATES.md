# ImUi Debug Draw Triangle Mesh v1 Evidence and Gates

Status: Closed.

## Evidence

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_draw.cpp`
- `crates/fret-core/src/scene/mod.rs`
- `crates/fret-core/src/scene/validate.rs`
- `crates/fret-core/src/scene/fingerprint.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/vertex_color.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/image.rs`
- `crates/fret-render-wgpu/src/renderer/tests.rs`
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`
- `docs/adr/0002-display-list.md`
- `docs/audits/imui-imgui-gap-audit-2026-04-22.md`

## Gates

```bash
cargo fmt --package fret-core --package fret-render-wgpu --package fret-ui-kit -- --check
cargo nextest run -p fret-core --no-fail-fast
cargo nextest run -p fret-render-wgpu --lib triangle_encodes --no-fail-fast
cargo nextest run -p fret-render-wgpu --lib --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui debug_draw_list_records_triangle_mesh_commands --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui debug_draw_options_default_to_clipped_canvas --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-debug-draw-triangle-mesh-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
