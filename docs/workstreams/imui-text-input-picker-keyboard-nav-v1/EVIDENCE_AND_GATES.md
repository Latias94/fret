# ImUi Text Input Picker Keyboard Navigation v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-04

## Repro

- Render a completion picker, type a query, move active candidate with ArrowDown, and commit it with
  Enter.
- Render a history picker, open it empty, move to first with ArrowDown, wrap to last with ArrowUp,
  and commit it with NumpadEnter.
- Render an empty completion picker and verify Enter still reaches the input submit command.

## Evidence Anchors

- `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`

## Gates

```bash
cargo fmt --package fret-ui-kit --package fret-imui
cargo check -p fret-ui-kit --tests
cargo nextest run -p fret-imui input_text_completion_picker_keyboard_navigation input_text_history_picker_keyboard_navigation input_text_picker_keyboard_navigation --no-fail-fast
cargo nextest run -p fret-imui models_text --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-text-input-picker-keyboard-nav-v1/WORKSTREAM.json
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
