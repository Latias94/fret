# ImUi Color Edit Popup Options v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-05

## Reference Evidence

- `repo-ref/imgui/imgui.h`: `ImGuiColorEditFlags_DefaultOptions_` documents the default RGB,
  Uint8, input RGB, and HueBar posture.
- `repo-ref/imgui/imgui_widgets.cpp`: `ColorEditOptionsPopup()`, `ColorPickerOptionsPopup()`,
  and `SetColorEditOptions()` are the Dear ImGui option/default reference points.
- `docs/workstreams/imui-color-edit-numeric-input-v1/CLOSEOUT_AUDIT_2026-05-04.md`: previous
  bounded slice that left option/default depth as a follow-on.

## Implementation Anchors

- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/src/controls/mod.rs`
- `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `docs/workstreams/imui-color-edit-popup-options-v1/CLOSEOUT_AUDIT_2026-05-05.md`

## Gates

```bash
cargo fmt --package fret-ui-editor
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-color-edit-popup-options-v1/WORKSTREAM.json
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
