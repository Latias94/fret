# ImUi Debug Draw Path Elliptical Arc Builder v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-05

## Reference Evidence

- `repo-ref/imgui/imgui.h`: `ImDrawList::PathEllipticalArcTo`.
- `repo-ref/imgui/imgui_draw.cpp`: Dear ImGui appends rotated elliptical arc points to the current
  path and then finishes through `PathStroke` / `PathFillConvex`.
- `crates/fret-core/src/vector_path.rs`: Fret's existing Canvas path vocabulary remains the final
  paint substrate; this lane only adds path-builder ergonomics above it.

## Implementation Anchors

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`

## Gates

```bash
cargo fmt --package fret-ui-kit -- --check
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-debug-draw-path-elliptical-arc-builder-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
