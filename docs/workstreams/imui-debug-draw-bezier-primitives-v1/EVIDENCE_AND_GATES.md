# ImUi Debug Draw Bezier Primitives v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-05

## Reference Evidence

- `repo-ref/imgui/imgui.h`: `ImDrawList::AddBezierQuadratic` and `AddBezierCubic`.
- `repo-ref/imgui/imgui_draw.cpp`: Dear ImGui lowers Bezier helpers through its path stroke flow.
- `repo-ref/imgui/imgui_demo.cpp`: debug draw demo shows quadratic and cubic Bezier examples.
- `crates/fret-core/src/vector_path.rs`: Fret already has `PathCommand::QuadTo` and
  `PathCommand::CubicTo`.

## Implementation Anchors

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`

## Gates

```bash
cargo fmt --package fret-ui-kit -- --check
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-debug-draw-bezier-primitives-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
