# ImUi Color Edit Alpha Preview Options v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-05

## Implementation Anchors

- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/preview.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/swatches.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`

## Reference Anchors

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_widgets.cpp`

## Gates

```bash
cargo fmt --package fret-ui-editor -- --check
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-color-edit-alpha-preview-options-v1/WORKSTREAM.json
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
