# ImUi Color Edit Numeric Input v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-04

## Reference Evidence

- `repo-ref/imgui/imgui_widgets.cpp`: Dear ImGui keeps RGB/HSV numeric edit/display inside the
  broader `ColorEdit4` / `ColorPicker4` family.
- `docs/workstreams/imui-color-edit-numeric-readout-v1/CLOSEOUT_AUDIT_2026-05-04.md`: previous
  bounded readout slice that left editable numeric input modes as follow-ons.

## Implementation Anchors

- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `docs/workstreams/imui-color-edit-numeric-input-v1/CLOSEOUT_AUDIT_2026-05-04.md`

## Gates

```bash
cargo fmt --package fret-ui-editor
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-color-edit-numeric-input-v1/WORKSTREAM.json
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
