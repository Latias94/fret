# ImUi Text Input Picker Recipe v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-04

## Repro

- Render a model-backed IMUI completion picker with app-owned candidates.
- Type a query and verify only matching candidates are mounted.
- Click a candidate and verify the text model, picked index, picked value, and changed signal.
- Render a history picker with empty input and verify unfiltered history entries mount.

## Evidence Anchors

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_widgets.cpp`
- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`

## Gates

```bash
cargo fmt --package fret-ui-kit --package fret-imui
cargo check -p fret-ui-kit --tests
cargo nextest run -p fret-imui input_text_completion_picker input_text_history_picker --no-fail-fast
cargo nextest run -p fret-imui models_text --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-text-input-picker-recipe-v1/WORKSTREAM.json
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
