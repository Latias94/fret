# ImUi Color Edit Model Split v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-05

## Reference Evidence

- `repo-ref/imgui/imgui_widgets.cpp`: `ColorEdit4()` and picker helpers demonstrate why color-edit
  depth becomes large quickly when model math and UI assembly are kept together.
- `docs/workstreams/imui-color-edit-popup-options-v1/CLOSEOUT_AUDIT_2026-05-05.md`: previous
  bounded feature slice that left the file large enough to justify a cleanup-only follow-on.

## Implementation Anchors

- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/model.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `docs/workstreams/imui-color-edit-model-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`

## Gates

```bash
cargo fmt --package fret-ui-editor -- --check
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-color-edit-model-split-v1/WORKSTREAM.json
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
