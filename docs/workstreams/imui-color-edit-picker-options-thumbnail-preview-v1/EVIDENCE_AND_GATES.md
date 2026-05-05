# ImUi Color Edit Picker Options Thumbnail Preview v1 Evidence and Gates

Status: Closed.

## Evidence

- `repo-ref/imgui/imgui_widgets.cpp`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/options.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `docs/workstreams/imui-color-edit-picker-options-thumbnail-preview-v1/CLOSEOUT_AUDIT_2026-05-05.md`

## Gates

```bash
cargo fmt --package fret-ui-editor -- --check
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
python -m json.tool docs/workstreams/imui-color-edit-picker-options-thumbnail-preview-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python tools/check_layering.py
python tools/check_workstream_catalog.py
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
