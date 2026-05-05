# ImUi Debug Draw Cookbook Proof v1 Evidence and Gates

Status: Closed.

## Evidence

- `apps/fret-cookbook/examples/imui_debug_draw_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `apps/fret-cookbook/Cargo.toml`
- `apps/fretboard/src/demos.rs`
- `apps/fret-cookbook/README.md`
- `apps/fret-cookbook/EXAMPLES.md`
- `docs/examples/README.md`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_draw.cpp`

## Gates

```bash
cargo fmt --package fret-cookbook -- --check
cargo build -p fret-cookbook --example imui_debug_draw_basics --features cookbook-imui
cargo nextest run -p fret-cookbook --lib cookbook_imui_debug_draw_example_keeps_current_facade_teaching_surface --no-fail-fast
cargo nextest run -p fretboard-dev cookbook_feature_hints_cover_imui_teaching_examples --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-debug-draw-cookbook-proof-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
