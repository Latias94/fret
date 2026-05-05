# ImUi Color Edit Reference Preview v1 Evidence and Gates

Status: Closed.

## Evidence

- `repo-ref/imgui/imgui_widgets.cpp`
- `repo-ref/imgui/imgui.h`
- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/preview.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs`
- `ecosystem/fret-ui-editor/src/controls/mod.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`

## Gates

```bash
cargo fmt --package fret-ui-editor -- --check
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
python -m json.tool docs/workstreams/imui-color-edit-reference-preview-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python tools/check_layering.py
python tools/check_workstream_catalog.py
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
