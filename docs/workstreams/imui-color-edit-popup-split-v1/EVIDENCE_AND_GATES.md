# ImUi Color Edit Popup Split v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-05

## Reference Evidence

- `repo-ref/imgui/imgui_widgets.cpp`: Dear ImGui's `ColorEdit4()` / picker implementation shows why
  popup composition grows quickly when it stays inline with public control wiring.
- `docs/workstreams/imui-color-edit-model-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`: previous
  cleanup-only split that moved pure model helpers before this popup composition split.

## Implementation Anchors

- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/model.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `docs/workstreams/imui-color-edit-popup-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`

## Gates

```bash
cargo fmt --package fret-ui-editor -- --check
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-color-edit-popup-split-v1/WORKSTREAM.json
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
