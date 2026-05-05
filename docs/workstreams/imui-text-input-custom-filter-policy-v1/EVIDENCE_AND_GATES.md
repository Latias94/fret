# ImUi Text Input Custom Filter Policy v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-04

## Repro

- Render a model-backed IMUI input with `InputTextOptions::filters` and
  `InputTextOptions::custom_filter`.
- Dispatch mixed text input.
- Assert the model reflects named-filter output followed by custom-filter output.

## Evidence Anchors

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_widgets.cpp`
- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`

## Gates

```bash
cargo fmt --package fret-ui-kit --package fret-imui
cargo check -p fret-ui-kit --tests
cargo nextest run -p fret-imui input_text_custom_filter input_text_named_filters input_text_numeric_filters --no-fail-fast
cargo nextest run -p fret-imui models_text --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-text-input-custom-filter-policy-v1/WORKSTREAM.json
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
