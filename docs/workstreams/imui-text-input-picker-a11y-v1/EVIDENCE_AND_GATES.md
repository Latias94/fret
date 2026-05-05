# ImUi Text Input Picker Accessibility v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-04

## Reference Evidence

- `crates/fret-ui/src/element.rs`: `TextInputProps` active-descendant / controls / expanded fields.
- `ecosystem/fret-ui-kit/src/headless/text_assist.rs`: existing input-owned text-assist semantics
  pattern used by editor controls.
- `ecosystem/fret-ui-editor/src/controls/text_assist_field.rs`: richer editor-owned text assist
  recipe; used as a policy reference, not copied into generic IMUI.

## Implementation Anchors

- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`
- `docs/workstreams/imui-text-input-picker-a11y-v1/CLOSEOUT_AUDIT_2026-05-04.md`

## Gates

```bash
cargo fmt --package fret-ui-kit --package fret-imui
cargo check -p fret-ui-kit --tests
cargo nextest run -p fret-imui input_text_completion_picker_keyboard_navigation input_text_history_picker_keyboard_navigation input_text_picker_keyboard_navigation --no-fail-fast
cargo nextest run -p fret-imui models_text --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-text-input-picker-a11y-v1/WORKSTREAM.json
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
