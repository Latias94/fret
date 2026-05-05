# ImUi Text Input Filter Policy v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-04

## Repro

- Focus an IMUI single-line input with `InputTextOptions::filters`.
- Dispatch `Event::TextInput` with mixed accepted/rejected characters.
- Assert the model only receives the filtered insertion text.

## Evidence Anchors

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_widgets.cpp`
- `crates/fret-ui/src/element.rs`
- `crates/fret-ui/src/text/input/widget.rs`
- `crates/fret-ui/src/text/input/tests.rs`
- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`

## Gates

```bash
cargo fmt --package fret-ui --package fret-ui-kit --package fret-imui
cargo nextest run -p fret-ui text_input_insert_filter --no-fail-fast
cargo nextest run -p fret-imui input_text_named_filters input_text_numeric_filters --no-fail-fast
cargo check -p fret-ui --tests
cargo check -p fret-ui-kit --tests
cargo nextest run -p fret-imui models_text --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-text-input-filter-policy-v1/WORKSTREAM.json
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
